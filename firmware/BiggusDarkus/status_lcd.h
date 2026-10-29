/// @file    status_lcd.h
/// @brief   16x2 I2C LCD: mode line, command text and live stats row.

#pragma once

#include <stdint.h>

void initStatusLcd();

// Write row 1 or 2, padded/truncated to 16 chars.
void showLcdRow(uint8_t row, const char* text);
void clearLcdRow(uint8_t row);
void clearStatusLcd();

// Call every loop in demo mode; updates the FPS and current row twice a second.
void loopStats();
