/// @file    four_digit_display.cpp
/// @brief   Clock, startwatch and timer on the 4-digit TM1637 display.

#include "four_digit_display.h"

#include <Arduino.h>
#include <TM1637TinyDisplay.h>

#include "pins.h"

namespace {

constexpr uint8_t kColonMask = 0b01000000;
constexpr uint32_t kSecondsPerMinute = 60;
constexpr uint32_t kSecondsPerHour = 60 * 60;
// UL suffix: plain ints are 16-bit on AVR, so 24 * 60 * 60 would overflow.
constexpr uint32_t kSecondsPerDay = 24UL * 60 * 60;

TM1637TinyDisplay display(PIN_CLK_4DIG, PIN_DIO_4DIG);  // 4 digits with colon

// Each mode keeps only its base value and the moment it started; no ticks.
struct ModeState {
  uint32_t baseSeconds;
  uint32_t startedAtMs;
  bool running;
};

ModeState states[3];
TimeMode current = TimeMode::Time;
uint32_t shownValue = 0xFFFFFFFFu;

ModeState& stateOf(TimeMode mode) {
  return states[static_cast<uint8_t>(mode)];
}

// Value of a mode at this moment: running modes advance, paused ones hold.
uint32_t valueOf(TimeMode mode, uint32_t now) {
  const ModeState& state = stateOf(mode);
  uint32_t elapsed = state.running ? (now - state.startedAtMs) / 1000 : 0;

  switch (mode) {
    case TimeMode::Time:
      return (state.baseSeconds + elapsed) % kSecondsPerDay;
    case TimeMode::Startwatch:
      return state.baseSeconds + elapsed;
    case TimeMode::Timer:
      return elapsed >= state.baseSeconds ? 0 : state.baseSeconds - elapsed;
  }
  return 0;
}

// 4 digits for the display: hh:mm for the clock, mm:ss below one hour otherwise.
uint32_t displayValue(TimeMode mode, uint32_t seconds) {
  if (mode == TimeMode::Time) {
    return ((seconds / kSecondsPerHour) % 24) * 100 + (seconds / kSecondsPerMinute) % 60;
  }
  if (seconds < kSecondsPerHour) {
    return (seconds / kSecondsPerMinute) * 100 + seconds % kSecondsPerMinute;
  }
  return ((seconds / kSecondsPerHour) % 100) * 100 + (seconds / kSecondsPerMinute) % 60;
}

void writeValue(uint32_t value) {
  display.showNumberDec(value, kColonMask, true);
  shownValue = value;
}

}  // namespace

void initFourDigitDisplay() {
  display.begin();
  display.setBrightness(3);

  for (uint8_t i = 0; i < 3; i++) {
    states[i].baseSeconds = 0;
    states[i].startedAtMs = millis();
    states[i].running = false;
  }
  // the clock always runs, from 00:00 at boot
  stateOf(TimeMode::Time).running = true;
  current = TimeMode::Time;
  shownValue = 0xFFFFFFFFu;
  loopFourDigitDisplay();
}

void loopFourDigitDisplay() {
  uint32_t value = displayValue(current, valueOf(current, millis()));
  if (value == shownValue) return;
  writeValue(value);
}

void selectTimeMode(TimeMode mode) {
  uint32_t now = millis();
  ModeState& state = stateOf(mode);
  if (!state.running) {
    state.startedAtMs = now;
    state.running = true;
  }
  current = mode;
  shownValue = 0xFFFFFFFFu;
  loopFourDigitDisplay();
}

void startTimeMode(TimeMode mode, uint32_t value) {
  if (mode == TimeMode::Time) value %= kSecondsPerDay;

  ModeState& state = stateOf(mode);
  state.baseSeconds = value;
  state.startedAtMs = millis();
  state.running = true;
  current = mode;
  shownValue = 0xFFFFFFFFu;
  loopFourDigitDisplay();
}

void pauseTimeMode(TimeMode mode) {
  if (mode == TimeMode::Time) return;

  uint32_t now = millis();
  ModeState& state = stateOf(mode);
  state.baseSeconds = valueOf(mode, now);
  state.running = false;
  current = mode;
  shownValue = 0xFFFFFFFFu;
  loopFourDigitDisplay();
}

void showFourDigitDemo() {
  writeValue(1234);  // 12:34
}

void blankFourDigitDisplay() {
  display.clear();
  shownValue = 0xFFFFFFFFu;
}
