// @filter: (memory is large)

/// @file    BiggusDarkus.ino
/// @brief   Demo mode switch for the LED strip, status LCD and digit displays

#include <Wire.h>
#include <LiquidCrystal_I2C.h>

#include "digit_displays.h"
#include "led_strip.h"
#include "pins.h"

// LCD Display
LiquidCrystal_I2C lcd(LCD_I2C_ADDR, 16, 2);  // 16 chars and 2 line display

void initLcd() {
  lcd.init();
  lcd.backlight();
}

// Button to ground; INPUT_PULLUP keeps the pin HIGH until pressed
#define DEBOUNCE_MS 50

bool demoOn = true;

// Show the mode on the LCD and blank everything else when the demo is off
void applyDemoMode() {
  lcd.clear();
  lcd.print(demoOn ? "Demo ON" : "Demo OFF");

  if (!demoOn) {
    clearLedStrip();
    clearDigitDisplays();
  }
}

// Toggles demoOn on each debounced press (HIGH -> LOW edge)
void loopButton() {
  static bool lastReading = HIGH;
  static bool stable = HIGH;
  static uint32_t lastChange = 0;

  bool reading = digitalRead(PIN_BTN);
  if (reading != lastReading) {
    lastReading = reading;
    lastChange = millis();
  }

  if (millis() - lastChange >= DEBOUNCE_MS && reading != stable) {
    stable = reading;
    if (stable == LOW) {
      demoOn = !demoOn;
      applyDemoMode();
    }
  }
}

// Shows the real frame rate and estimated strip current on LCD row 2
void loopStats() {
  static uint16_t frames = 0;
  static uint32_t lastUpdate = 0;

  frames++;

  // refresh twice a second: LCD writes are slow and would eat into the frame rate
  uint32_t elapsed = millis() - lastUpdate;
  if (elapsed < 500) return;

  uint16_t fps = (uint32_t)frames * 1000 / elapsed;
  LedStripPower power = readLedStripPower();

  // padded to 16 chars so it overwrites the previous text without clearing the row
  char line[17];
  snprintf(line, sizeof(line), "FPS%4u %c%4umA ", fps, power.limited ? '>' : ' ',
           (unsigned)power.milliAmps);
  lcd.setCursor(0, 1);
  lcd.print(line);

  frames = 0;
  lastUpdate = millis();
}

void setup() {
  initLcd();

  initDigitDisplays();

  pinMode(PIN_BTN, INPUT_PULLUP);

  delay(3000); // 3 second delay for recovery

  initLedStrip();

  applyDemoMode();
}

void loop() {
  loopButton();

  // Off: strips and displays were blanked once in applyDemoMode(), so just idle
  if (!demoOn) return;

  showLedDemoFrame();
  showDigitDemo();
  loopStats();
}
