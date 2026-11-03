/// @file    led_strip.cpp
/// @brief   FastLED patterns, command overlays and power estimation.
/// @note    Patterns are based on the FastLED DemoReel100 example by Mark Kriegsman.

#include "led_strip.h"

#include <FastLED.h>

#include "pins.h"

namespace {

constexpr uint8_t kBrightness = 60;       // keep low to save energy
constexpr uint8_t kFramesPerSecond = 120;
constexpr uint16_t kMaxCurrentMa = 400;   // sized to keep USB-only runs inside the 500 mA port limit

CRGB leds[kLedCount];

uint8_t currentPattern = 0;  // index into patterns
uint8_t hue = 0;             // rotating "base color" used by many of the patterns

// Each LED remembers its own command state, so a new command simply overwrites it.
struct LedState {
  bool on;
  CRGB color;
  LedAnimation animation;
  uint16_t periodMs;
};

LedState states[kLedCount];

void rainbow() {
  // FastLED's built-in rainbow generator
  fill_rainbow(leds, kLedCount, hue, 7);
}

void addGlitter(fract8 chanceOfGlitter) {
  if (random8() < chanceOfGlitter) {
    leds[random16(kLedCount)] += CRGB::White;
  }
}

void rainbowWithGlitter() {
  // built-in FastLED rainbow, plus some random sparkly glitter
  rainbow();
  addGlitter(80);
}

void confetti() {
  // random colored speckles that blink in and fade smoothly
  fadeToBlackBy(leds, kLedCount, 10);
  int pos = random16(kLedCount);
  leds[pos] += CHSV(hue + random8(64), 200, 255);
}

void sinelon() {
  // a colored dot sweeping back and forth, with fading trails
  fadeToBlackBy(leds, kLedCount, 20);
  int pos = beatsin16(13, 0, kLedCount - 1);
  leds[pos] += CHSV(hue, 255, 192);
}

void bpm() {
  // colored stripes pulsing at a defined Beats-Per-Minute (BPM)
  uint8_t beatsPerMinute = 62;
  CRGBPalette16 palette = PartyColors_p;
  uint8_t beat = beatsin8(beatsPerMinute, 64, 255);
  for (int i = 0; i < kLedCount; i++) {
    leds[i] = ColorFromPalette(palette, hue + (i * 2), beat - hue + (i * 10));
  }
}

void juggle() {
  // eight colored dots, weaving in and out of sync with each other
  fadeToBlackBy(leds, kLedCount, 20);
  uint8_t dothue = 0;
  for (int i = 0; i < 8; i++) {
    leds[beatsin16(i + 7, 0, kLedCount - 1)] |= CHSV(dothue, 200, 255);
    dothue += 32;
  }
}

using Pattern = void (*)();

Pattern patterns[] = {rainbow, rainbowWithGlitter, confetti, sinelon, juggle, bpm};

void nextPattern() {
  currentPattern = (currentPattern + 1) % (sizeof(patterns) / sizeof(patterns[0]));
}

void paintLed(uint8_t index, uint32_t now) {
  const LedState& state = states[index];
  if (!state.on) return;

  uint16_t period = state.periodMs == 0 ? 500 : state.periodMs;
  CRGB color = state.color;

  if (state.animation == LedAnimation::Blink) {
    if (((now / period) & 1) != 0) return;
  } else if (state.animation == LedAnimation::Pulse) {
    // triangle wave: dark at the edges, full brightness in the middle
    uint32_t phase = (now % period) * 255UL / period;
    uint8_t scale = phase < 128 ? phase * 2 : (255 - phase) * 2;
    color.nscale8(scale);
  }

  leds[index] = color;
}

void drawPattern() {
  patterns[currentPattern]();
}

void paintStates(uint32_t now) {
  for (uint8_t i = 0; i < kLedCount; i++) paintLed(i, now);
}

void advancePattern() {
  EVERY_N_MILLISECONDS(20) { hue++; }       // slowly cycle the "base color" through the rainbow
  EVERY_N_SECONDS(10) { nextPattern(); }    // change patterns periodically
}

}  // namespace

void initLedStrip() {
  clearLedRanges();
  FastLED.addLeds<WS2811, PIN_LED_DATA, GRB>(leds, kLedCount).setCorrection(TypicalLEDStrip);
  FastLED.setBrightness(kBrightness);
  // limit current to save energy for a green planet
  FastLED.setMaxPowerInVoltsAndMilliamps(5, kMaxCurrentMa);
}

void showLedDemoFrame() {
  drawPattern();
  FastLED.show();
  // insert a delay to keep the framerate modest
  FastLED.delay(1000 / kFramesPerSecond);
  advancePattern();
}

void showActiveLedFrame() {
  // active mode shows only command states; the demo pattern stays hidden
  FastLED.clear();
  paintStates(millis());
  FastLED.show();
  FastLED.delay(1000 / kFramesPerSecond);
}

void clearLedStrip() {
  FastLED.clear(true);  // true = push the black frame to the strip right away
}

LedStripPower readLedStripPower() {
  // the library gives power at full brightness (assuming 5V), so scale it by our brightness
  uint32_t mA = calculate_unscaled_power_mW(leds, kLedCount) / 5 * kBrightness / 256;
  // above the cap FastLED dims to exactly kMaxCurrentMa, so report the clamped value
  bool limited = mA > kMaxCurrentMa;
  return {limited ? kMaxCurrentMa : (uint16_t)mA, limited};
}

void setLedRange(uint8_t start, uint8_t end, uint8_t red, uint8_t green, uint8_t blue,
                 LedAnimation animation, uint16_t periodMs) {
  if (start >= end || end > kLedCount) return;
  for (uint8_t i = start; i < end; i++) {
    states[i] = {true, CRGB(red, green, blue), animation, periodMs};
  }
}

void clearLedRange(uint8_t start, uint8_t end) {
  for (uint8_t i = start; i < end && i < kLedCount; i++) states[i].on = false;
}

void clearLedRanges() {
  clearLedRange(0, kLedCount);
}
