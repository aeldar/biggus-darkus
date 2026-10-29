// @filter: (memory is large)

/// @file    BiggusDarkus.ino
/// @brief   Demo mode switch for the LED strip, status LCD and digit displays

#include "button.h"
#include "counter_display.h"
#include "led_strip.h"
#include "status_lcd.h"
#include "stopwatch_display.h"

bool demoOn = true;

// Show the mode on the LCD and blank everything else when the demo is off
void applyDemoMode() {
  showDemoMode(demoOn);

  if (!demoOn) {
    clearLedStrip();
    clearStopwatchDisplay();
    clearCounterDisplay();
  }
}

void setup() {
  initStatusLcd();

  initStopwatchDisplay();
  initCounterDisplay();

  initButton();

  delay(3000); // 3 second delay for recovery

  initLedStrip();

  applyDemoMode();
}

void loop() {
  if (buttonPressed()) {
    demoOn = !demoOn;
    applyDemoMode();
  }

  // Off: strips and displays were blanked once in applyDemoMode(), so just idle
  if (!demoOn) return;

  showLedDemoFrame();
  showStopwatchDemo();
  showCounterDemo();
  loopStats();
}
