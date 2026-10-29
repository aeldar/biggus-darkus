/// @file    stopwatch_display.h
/// @brief   4-digit TM1637 stopwatch display.

#pragma once

void initStopwatchDisplay();

// Call every loop; the display refreshes at its own pace.
void showStopwatchDemo();

void clearStopwatchDisplay();
