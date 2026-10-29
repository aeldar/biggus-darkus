/// @file    led_strip.h
/// @brief   WS2811 strip setup, demo patterns, command overlays and power estimation.

#pragma once

#include <stdint.h>

constexpr uint8_t kLedCount = 23 * 2;

enum class LedAnimation : uint8_t { Solid, Blink, Pulse };

// Estimated strip draw, clamped to the configured limit.
struct LedStripPower {
  uint16_t milliAmps;
  bool limited;  // true when the estimate was above the limit and got clamped
};

void initLedStrip();

// Advance the pattern rotation and push one demo frame.
void showLedDemoFrame();

// Paint the active-mode frame: black plus any command overlays.
void showActiveLedFrame();

void clearLedStrip();

LedStripPower readLedStripPower();

// Paint [start, end) with a colour and animation. False when all slots are full.
bool setLedRange(uint8_t start, uint8_t end, uint8_t red, uint8_t green, uint8_t blue,
                 LedAnimation animation, uint16_t periodMs);

void clearLedRange(uint8_t start, uint8_t end);
void clearLedRanges();
