/// @file    counter_display.cpp
/// @brief   Static and counting modes on the 6-digit TM1637 display.

#include "counter_display.h"

#include <Arduino.h>
#include <TM1637TinyDisplay6.h>

#include "pins.h"

namespace {

constexpr long kCounterMax = 999999;
constexpr uint16_t kDefaultPeriodMs = 100;

TM1637TinyDisplay6 display(PIN_CLK_6DIG, PIN_DIO_6DIG);  // 6 digits

enum class CounterMode : uint8_t { Static, CountUp, CountDown };

CounterMode mode = CounterMode::Static;
long baseValue = 0;
uint32_t startedAtMs = 0;
uint16_t periodMs = kDefaultPeriodMs;
long shownValue = -1;

// Value at this moment; counting modes derive it from the start time, not ticks.
long valueOf(uint32_t now) {
  if (mode == CounterMode::Static) return baseValue;

  uint32_t steps = (now - startedAtMs) / periodMs;
  if (mode == CounterMode::CountUp) {
    if (steps >= (uint32_t)(kCounterMax - baseValue)) return kCounterMax;
    return baseValue + (long)steps;
  }
  if (steps >= (uint32_t)baseValue) return 0;
  return baseValue - (long)steps;
}

void writeValue(long value) {
  display.showNumber(value);
  shownValue = value;
}

}  // namespace

void initCounterDisplay() {
  display.begin();
  display.setBrightness(3);

  mode = CounterMode::Static;
  baseValue = 0;
  shownValue = -1;
  writeValue(0);
}

void loopCounterDisplay() {
  long value = valueOf(millis());
  if (value != shownValue) writeValue(value);
}

void setCounter(long value) {
  mode = CounterMode::Static;
  baseValue = constrain(value, 0L, kCounterMax);
  writeValue(baseValue);
}

void resetCounter() {
  setCounter(0);
}

void startCounterUp(long start, uint16_t intervalMs) {
  mode = CounterMode::CountUp;
  baseValue = constrain(start, 0L, kCounterMax);
  startedAtMs = millis();
  periodMs = intervalMs == 0 ? kDefaultPeriodMs : intervalMs;
  writeValue(baseValue);
}

void startCounterDown(long start, uint16_t intervalMs) {
  mode = CounterMode::CountDown;
  baseValue = constrain(start, 0L, kCounterMax);
  startedAtMs = millis();
  periodMs = intervalMs == 0 ? kDefaultPeriodMs : intervalMs;
  writeValue(baseValue);
}

void showCounterDemo() {
  writeValue(123456);
}

void blankCounterDisplay() {
  display.clear();
  shownValue = -1;
}
