/// @file    status_lcd.cpp
/// @brief   Demo mode line and live strip stats on the 16x2 LCD.

#include "status_lcd.h"

#include <Arduino.h>
#include <Wire.h>
#include <LiquidCrystal_I2C.h>

#include "led_strip.h"
#include "pins.h"

namespace {

LiquidCrystal_I2C lcd(LCD_I2C_ADDR, 16, 2);  // 16 chars and 2 line display

}  // namespace

void initStatusLcd() {
  lcd.init();
  lcd.backlight();
}

void showDemoMode(bool on) {
  lcd.clear();
  lcd.print(on ? "Demo ON" : "Demo OFF");
}

void loopStats() {
  static uint16_t frames = 0;
  static uint32_t lastUpdate = 0;

  frames++;

  // refresh twice a second: LCD writes are slow and would eat into the frame rate
  uint32_t elapsed = millis() - lastUpdate;
  if (elapsed < 500) return;

  uint16_t fps = (uint32_t)frames * 1000 / elapsed;
  LedStripPower power = readLedStripPower();

  // padded to 16 chars so it overwrites the previous text without clearing the row
  char line[17];
  snprintf(line, sizeof(line), "FPS%4u %c%4umA ", fps, power.limited ? '>' : ' ',
           (unsigned)power.milliAmps);
  lcd.setCursor(0, 1);
  lcd.print(line);

  frames = 0;
  lastUpdate = millis();
}
