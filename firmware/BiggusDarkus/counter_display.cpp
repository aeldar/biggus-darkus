/// @file    counter_display.cpp
/// @brief   Running counter shown on the 6-digit TM1637 display.

#include "counter_display.h"

#include <Arduino.h>
#include <TM1637TinyDisplay6.h>

#include "pins.h"

namespace {

constexpr uint32_t kUpdateIntervalMs = 100;

TM1637TinyDisplay6 display(PIN_CLK_6DIG, PIN_DIO_6DIG);  // 6 digits

long counter = 0;
uint32_t lastUpdate = 0;

}  // namespace

void initCounterDisplay() {
  display.begin();
  display.setBrightness(3);
}

void showCounterDemo() {
  // non-blocking: delay() here would stall the LED animation too
  if (millis() - lastUpdate < kUpdateIntervalMs) return;
  lastUpdate = millis();

  display.showNumber(counter++);
}

void clearCounterDisplay() {
  display.clear();
}
