/// @file    stopwatch_display.cpp
/// @brief   Stopwatch shown on the 4-digit TM1637 display.

#include "stopwatch_display.h"

#include <Arduino.h>
#include <TM1637TinyDisplay.h>

#include "pins.h"

namespace {

constexpr uint8_t kColonMask = 0b01000000;

TM1637TinyDisplay display(PIN_CLK_4DIG, PIN_DIO_4DIG);  // 4 digits with colon

bool started = false;
uint32_t startMs = 0;
uint32_t shownSeconds = 0;

// Show mm:ss up to 59:59, then hh:mm; wraps at 99:59.
void showTime(uint32_t seconds) {
  uint32_t totalMinutes = seconds / 60;
  uint32_t value;
  if (totalMinutes < 60) {
    value = totalMinutes * 100 + seconds % 60;
  } else {
    value = (totalMinutes / 60) % 100 * 100 + totalMinutes % 60;
  }
  display.showNumberDec(value, kColonMask, true);
}

}  // namespace

void initStopwatchDisplay() {
  display.begin();
  display.setBrightness(3);
  showTime(0);
}

void showStopwatchDemo() {
  uint32_t now = millis();

  // start on the first demo frame so the stopwatch visibly begins at 00:00
  if (!started) {
    started = true;
    startMs = now;
    showTime(0);
  }

  uint32_t elapsed = (now - startMs) / 1000;
  if (elapsed != shownSeconds) {
    shownSeconds = elapsed;
    showTime(elapsed);
  }
}

void clearStopwatchDisplay() {
  display.clear();
  // force the next frame to redraw even if the second has not changed
  shownSeconds = 0xFFFFFFFFu;
}
