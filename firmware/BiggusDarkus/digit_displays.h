/// @file    digit_displays.h
/// @brief   Demo behaviour for the 4-digit clock and 6-digit counter displays.

#pragma once

void initDigitDisplays();

// Call every loop; the displays refresh at their own pace.
void showDigitDemo();

void clearDigitDisplays();
