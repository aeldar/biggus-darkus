/// @file    counter_display.h
/// @brief   6-digit TM1637 running counter display.

#pragma once

void initCounterDisplay();

// Call every loop; the display refreshes at its own pace.
void showCounterDemo();

void clearCounterDisplay();
