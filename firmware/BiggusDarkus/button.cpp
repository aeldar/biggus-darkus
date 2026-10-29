/// @file    button.cpp
/// @brief   Debounced press detection for the mode button.

#include "button.h"

#include <Arduino.h>

#include "pins.h"

namespace {

constexpr uint32_t kDebounceMs = 50;

bool lastReading = HIGH;
bool stable = HIGH;
uint32_t lastChange = 0;

}  // namespace

void initButton() {
  // Button to ground; INPUT_PULLUP keeps the pin HIGH until pressed
  pinMode(PIN_BTN, INPUT_PULLUP);
}

bool buttonPressed() {
  bool reading = digitalRead(PIN_BTN);
  if (reading != lastReading) {
    lastReading = reading;
    lastChange = millis();
  }

  if (millis() - lastChange >= kDebounceMs && reading != stable) {
    stable = reading;
    return stable == LOW;
  }

  return false;
}
