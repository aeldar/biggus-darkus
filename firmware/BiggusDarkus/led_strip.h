/// @file    led_strip.h
/// @brief   WS2811 strip setup, demo patterns and power estimation.

#pragma once

#include <stdint.h>

// Estimated strip draw, clamped to the configured limit.
struct LedStripPower {
  uint16_t milliAmps;
  bool limited;  // true when the estimate was above the limit and got clamped
};

void initLedStrip();

// Advance the pattern rotation and push one frame to the strip.
void showLedDemoFrame();

void clearLedStrip();

LedStripPower readLedStripPower();
