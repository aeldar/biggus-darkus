/// @file    digit_displays.cpp
/// @brief   TM1637 stopwatch and counter demo.

#include "digit_displays.h"

#include <Arduino.h>
#include <TM1637TinyDisplay.h>
#include <TM1637TinyDisplay6.h>

#include "pins.h"

namespace {

constexpr uint8_t kColonMask = 0b01000000;
constexpr uint32_t kCounterIntervalMs = 100;

TM1637TinyDisplay clockDisplay(PIN_CLK_4DIG, PIN_DIO_4DIG);  // 4 digits with colon
TM1637TinyDisplay6 bigDisplay(PIN_CLK_6DIG, PIN_DIO_6DIG);    // 6 digits

long counter = 0;
uint32_t lastCounterUpdate = 0;

bool stopwatchStarted = false;
uint32_t stopwatchStartMs = 0;
uint32_t stopwatchShownSeconds = 0;

// Show mm:ss up to 59:59, then hh:mm; wraps at 99:59.
void showStopwatch(uint32_t seconds) {
  uint32_t totalMinutes = seconds / 60;
  uint32_t value;
  if (totalMinutes < 60) {
    value = totalMinutes * 100 + seconds % 60;
  } else {
    value = (totalMinutes / 60) % 100 * 100 + totalMinutes % 60;
  }
  clockDisplay.showNumberDec(value, kColonMask, true);
}

}  // namespace

void initDigitDisplays() {
  clockDisplay.begin();
  bigDisplay.begin();
  clockDisplay.setBrightness(3);
  bigDisplay.setBrightness(3);
  showStopwatch(0);
}

void showDigitDemo() {
  uint32_t now = millis();

  // start on the first demo frame so the stopwatch visibly begins at 00:00
  if (!stopwatchStarted) {
    stopwatchStarted = true;
    stopwatchStartMs = now;
    showStopwatch(0);
  }

  uint32_t elapsed = (now - stopwatchStartMs) / 1000;
  if (elapsed != stopwatchShownSeconds) {
    stopwatchShownSeconds = elapsed;
    showStopwatch(elapsed);
  }

  // non-blocking: delay() here would stall the LED animation too
  if (now - lastCounterUpdate < kCounterIntervalMs) return;
  lastCounterUpdate = now;

  bigDisplay.showNumber(counter++);
}

void clearDigitDisplays() {
  clockDisplay.clear();
  bigDisplay.clear();
  // force the next frame to redraw the stopwatch even if the second has not changed
  stopwatchShownSeconds = 0xFFFFFFFFu;
}
