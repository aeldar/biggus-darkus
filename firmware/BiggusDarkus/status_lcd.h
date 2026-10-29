/// @file    status_lcd.h
/// @brief   16x2 I2C LCD: demo mode line and live stats row.

#pragma once

void initStatusLcd();

// Show "Demo ON" / "Demo OFF" on row 1.
void showDemoMode(bool on);

// Call every loop; updates the FPS and current row twice a second.
void loopStats();
