/// @file    digit_displays.cpp
/// @brief   TM1637 clock and counter demo.

#include "digit_displays.h"

#include <Arduino.h>
#include <TM1637TinyDisplay.h>
#include <TM1637TinyDisplay6.h>

#include "pins.h"

namespace {

TM1637TinyDisplay clockDisplay(PIN_CLK_4DIG, PIN_DIO_4DIG);  // 4 digits with colon
TM1637TinyDisplay6 bigDisplay(PIN_CLK_6DIG, PIN_DIO_6DIG);    // 6 digits

long counter = 0;
uint32_t lastUpdate = 0;

}  // namespace

void initDigitDisplays() {
  clockDisplay.begin();
  bigDisplay.begin();
  clockDisplay.setBrightness(3);
  bigDisplay.setBrightness(3);
}

void showDigitDemo() {
  // non-blocking: delay() here would stall the LED animation too
  if (millis() - lastUpdate < 100) return;
  lastUpdate = millis();

  // 12:34 with the colon on
  clockDisplay.showNumberDec(1234, 0b01000000, true);

  bigDisplay.showNumber(counter++);
}

void clearDigitDisplays() {
  clockDisplay.clear();
  bigDisplay.clear();
}
