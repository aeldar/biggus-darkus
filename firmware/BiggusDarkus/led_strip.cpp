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
constexpr uint8_t kMaxOverlays = 4;

CRGB leds[kLedCount];

uint8_t currentPattern = 0;  // index into patterns
uint8_t hue = 0;             // rotating "base color" used by many of the patterns

struct Overlay {
  bool active;
  uint8_t start;
  uint8_t end;
  CRGB color;
  LedAnimation animation;
  uint16_t periodMs;
};

Overlay overlays[kMaxOverlays];

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

void paintOverlay(const Overlay& overlay, uint32_t now) {
  uint16_t period = overlay.periodMs == 0 ? 500 : overlay.periodMs;
  CRGB color = overlay.color;

  if (overlay.animation == LedAnimation::Blink) {
    if (((now / period) & 1) != 0) return;
  } else if (overlay.animation == LedAnimation::Pulse) {
    // triangle wave: dark at the edges, full brightness in the middle
    uint32_t phase = (now % period) * 255UL / period;
    uint8_t scale = phase < 128 ? phase * 2 : (255 - phase) * 2;
    color.nscale8(scale);
  }

  for (uint8_t i = overlay.start; i < overlay.end; i++) {
    leds[i] = color;
  }
}

void drawPattern() {
  patterns[currentPattern]();
}

void paintOverlays(uint32_t now) {
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    if (overlays[i].active) paintOverlay(overlays[i], now);
  }
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
  // active mode shows only command overlays; the demo pattern stays hidden
  FastLED.clear();
  paintOverlays(millis());
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

bool setLedRange(uint8_t start, uint8_t end, uint8_t red, uint8_t green, uint8_t blue,
                 LedAnimation animation, uint16_t periodMs) {
  if (start >= end || end > kLedCount) return false;

  // slots we can reuse: idle ones plus overlays the new range fully covers
  uint8_t free = 0, splits = 0;
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    const Overlay& o = overlays[i];
    if (!o.active || (o.start >= start && o.end <= end)) free++;
    else if (o.start < start && o.end > end) splits++;
  }
  if (splits + 1 > free) return false;

  // the new command owns its range, so cut it out of every older overlay;
  // drop covered ones first so splits always find a free slot
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    if (overlays[i].active && overlays[i].start >= start && overlays[i].end <= end) {
      overlays[i].active = false;
    }
  }
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    Overlay& o = overlays[i];
    if (!o.active || o.end <= start || o.start >= end) continue;
    if (o.start < start && o.end > end) {
      Overlay right = o;
      right.start = end;
      for (uint8_t j = 0; j < kMaxOverlays; j++) {
        if (!overlays[j].active) { overlays[j] = right; break; }
      }
      o.end = start;
    } else if (o.start < start) {
      o.end = start;
    } else if (o.end > end) {
      o.start = end;
    }
  }

  Overlay* slot = nullptr;
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    if (!overlays[i].active) { slot = &overlays[i]; break; }
  }

  slot->active = true;
  slot->start = start;
  slot->end = end;
  slot->color = CRGB(red, green, blue);
  slot->animation = animation;
  slot->periodMs = periodMs;
  return true;
}

void clearLedRange(uint8_t start, uint8_t end) {
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    if (overlays[i].active && overlays[i].start == start && overlays[i].end == end) {
      overlays[i].active = false;
    }
  }
}

void clearLedRanges() {
  for (uint8_t i = 0; i < kMaxOverlays; i++) {
    overlays[i].active = false;
  }
}
