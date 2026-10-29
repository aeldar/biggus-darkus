/// @file    counter_display.h
/// @brief   6-digit TM1637: static, count-up and count-down modes.

#pragma once

#include <stdint.h>

void initCounterDisplay();

// Render/tick the active mode; call every loop.
void loopCounterDisplay();

void setCounter(long value);
void resetCounter();

void startCounterUp(long start, uint16_t periodMs);
void startCounterDown(long start, uint16_t periodMs);

// Static demo value, independent of the active state.
void showCounterDemo();

// Clear the display only; active state keeps running.
void blankCounterDisplay();
