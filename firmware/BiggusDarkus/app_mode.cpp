/// @file    app_mode.cpp
/// @brief   Mode transitions and the side effects they have on the hardware.

#include "app_mode.h"

#include "counter_display.h"
#include "four_digit_display.h"
#include "led_strip.h"
#include "status_lcd.h"

namespace {

Mode mode = Mode::Demo;

}  // namespace

void initMode() {
  setMode(Mode::Demo);
}

Mode currentMode() {
  return mode;
}

void setMode(Mode next) {
  mode = next;

  switch (mode) {
    case Mode::Off:
      blankFourDigitDisplay();
      blankCounterDisplay();
      clearLedStrip();
      clearStatusLcd();
      break;

    case Mode::Demo:
      clearLedStrip();  // drop the previous frame so the pattern starts clean
      showFourDigitDemo();
      showCounterDemo();
      showLcdRow(1, "Demo ON");
      clearLcdRow(2);
      break;

    case Mode::Active:
      showLcdRow(1, "Mode ACTIVE");
      clearLcdRow(2);
      break;
  }
}

void nextMode() {
  switch (mode) {
    case Mode::Demo:
      setMode(Mode::Active);
      break;
    case Mode::Active:
      setMode(Mode::Off);
      break;
    case Mode::Off:
      setMode(Mode::Demo);
      break;
  }
}
