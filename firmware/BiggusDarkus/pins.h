/// @file    pins.h
/// @brief   Board wiring for the Arduino Nano. The only place pin assignments
///          live: firmware and docs/hardware.md both point here.

#pragma once

// LED strip (WS2811 data line)
#define PIN_LED_DATA   A3

// Mode button, wired to ground; INPUT_PULLUP keeps it HIGH until pressed
#define PIN_BTN        A0

// 4-digit TM1637 (time display: clock, startwatch, timer)
#define PIN_CLK_4DIG   5
#define PIN_DIO_4DIG   6

// 6-digit TM1637 (counter display)
#define PIN_CLK_6DIG   7
#define PIN_DIO_6DIG   8

// I2C address of the 16x2 LCD backpack
#define LCD_I2C_ADDR   0x27
