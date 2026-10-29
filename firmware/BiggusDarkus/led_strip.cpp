/// @file    led_strip.cpp
/// @brief   FastLED "DemoReel100" pattern rotation.
/// @note    Patterns are based on the FastLED DemoReel100 example by Mark Kriegsman.

#include "led_strip.h"

#include <FastLED.h>

#include "pins.h"

namespace {

constexpr uint8_t kNumLeds = 23 * 2;
constexpr uint8_t kBrightness = 60;       // keep low to save energy
constexpr uint8_t kFramesPerSecond = 120;
constexpr uint16_t kMaxCurrentMa = 400;   // sized to keep USB-only runs inside the 500 mA port limit

CRGB leds[kNumLeds];

uint8_t currentPattern = 0;  // index into patterns
uint8_t hue = 0;             // rotating "base color" used by many of the patterns

void rainbow() {
  // FastLED's built-in rainbow generator
  fill_rainbow(leds, kNumLeds, hue, 7);
}

void addGlitter(fract8 chanceOfGlitter) {
  if (random8() < chanceOfGlitter) {
    leds[random16(kNumLeds)] += CRGB::White;
  }
}

void rainbowWithGlitter() {
  // built-in FastLED rainbow, plus some random sparkly glitter
  rainbow();
  addGlitter(80);
}

void confetti() {
  // random colored speckles that blink in and fade smoothly
  fadeToBlackBy(leds, kNumLeds, 10);
  int pos = random16(kNumLeds);
  leds[pos] += CHSV(hue + random8(64), 200, 255);
}

void sinelon() {
  // a colored dot sweeping back and forth, with fading trails
  fadeToBlackBy(leds, kNumLeds, 20);
  int pos = beatsin16(13, 0, kNumLeds - 1);
  leds[pos] += CHSV(hue, 255, 192);
}

void bpm() {
  // colored stripes pulsing at a defined Beats-Per-Minute (BPM)
  uint8_t beatsPerMinute = 62;
  CRGBPalette16 palette = PartyColors_p;
  uint8_t beat = beatsin8(beatsPerMinute, 64, 255);
  for (int i = 0; i < kNumLeds; i++) {
    leds[i] = ColorFromPalette(palette, hue + (i * 2), beat - hue + (i * 10));
  }
}

void juggle() {
  // eight colored dots, weaving in and out of sync with each other
  fadeToBlackBy(leds, kNumLeds, 20);
  uint8_t dothue = 0;
  for (int i = 0; i < 8; i++) {
    leds[beatsin16(i + 7, 0, kNumLeds - 1)] |= CHSV(dothue, 200, 255);
    dothue += 32;
  }
}

using Pattern = void (*)();

Pattern patterns[] = {rainbow, rainbowWithGlitter, confetti, sinelon, juggle, bpm};

void nextPattern() {
  currentPattern = (currentPattern + 1) % (sizeof(patterns) / sizeof(patterns[0]));
}

}  // namespace

void initLedStrip() {
  FastLED.addLeds<WS2811, PIN_LED_DATA, GRB>(leds, kNumLeds).setCorrection(TypicalLEDStrip);
  FastLED.setBrightness(kBrightness);
  // limit current to save energy for a green planet
  FastLED.setMaxPowerInVoltsAndMilliamps(5, kMaxCurrentMa);
}

void showLedDemoFrame() {
  patterns[currentPattern]();
  FastLED.show();
  // insert a delay to keep the framerate modest
  FastLED.delay(1000 / kFramesPerSecond);

  EVERY_N_MILLISECONDS(20) { hue++; }  // slowly cycle the "base color" through the rainbow
  EVERY_N_SECONDS(10) { nextPattern(); }  // change patterns periodically
}

void clearLedStrip() {
  FastLED.clear(true);  // true = push the black frame to the strip right away
}

LedStripPower readLedStripPower() {
  // the library gives power at full brightness (assuming 5V), so scale it by our brightness
  uint32_t mA = calculate_unscaled_power_mW(leds, kNumLeds) / 5 * kBrightness / 256;
  // above the cap FastLED dims to exactly kMaxCurrentMa, so report the clamped value
  bool limited = mA > kMaxCurrentMa;
  return {limited ? kMaxCurrentMa : (uint16_t)mA, limited};
}
