/// @file    four_digit_display.h
/// @brief   4-digit TM1637: clock, startwatch and timer modes.

#pragma once

#include <stdint.h>

enum class TimeMode : uint8_t { Time, Startwatch, Timer };

void initFourDigitDisplay();

// Render/tick the active mode; call every loop.
void loopFourDigitDisplay();

// Switch to a mode and resume it; a mode never started begins at 00:00.
void selectTimeMode(TimeMode mode);

// Set a mode's initial value and run it. Time is seconds of day.
void startTimeMode(TimeMode mode, uint32_t value);

// Switch to a mode and freeze it at its current value.
void pauseTimeMode(TimeMode mode);

// Static demo value, independent of the active state.
void showFourDigitDemo();

// Clear the display only; active state keeps running.
void blankFourDigitDisplay();
