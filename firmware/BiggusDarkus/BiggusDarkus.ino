// @filter: (memory is large)

/// @file    BiggusDarkus.ino
/// @brief   Demo and manual control for the LED strip, status LCD and digit displays

#include "app_mode.h"
#include "button.h"
#include "counter_display.h"
#include "four_digit_display.h"
#include "led_strip.h"
#include "serial_commands.h"
#include "status_lcd.h"

void setup() {
  initSerialCommands();

  initStatusLcd();

  initFourDigitDisplay();
  initCounterDisplay();

  initButton();

  delay(3000); // 3 second delay for recovery

  initLedStrip();

  initMode();  // start in DEMO
}

void loop() {
  loopSerialCommands();

  if (buttonPressed()) nextMode();

  switch (currentMode()) {
    case Mode::Off:
      break;

    case Mode::Demo:
      showLedDemoFrame();
      loopStats();
      break;

    case Mode::Active:
      showActiveLedFrame();
      loopFourDigitDisplay();
      loopCounterDisplay();
      break;
  }
}
