// @filter: (memory is large)

/// @file    BiggusDarkus.ino
/// @brief   LED strip effects with an LCD status readout and two TM1637 displays
/// @note    Effect patterns are based on the FastLED DemoReel100 example

#include <FastLED.h>

#include "pins.h"

//YWROBOT
//Compatible with the Arduino IDE 1.0
//Library version:1.1
#include <Wire.h> 
#include <LiquidCrystal_I2C.h>

// Digit Displays
#include <TM1637TinyDisplay.h>
#include <TM1637TinyDisplay6.h>

// FastLED "100-lines-of-code" demo reel, showing just a few 
// of the kinds of animation patterns you can quickly and easily 
// compose using FastLED.  
//
// This example also shows one easy way to define multiple 
// animations patterns and have them automatically rotate.
//
// -Mark Kriegsman, December 2014


#define LED_TYPE    WS2811
#define COLOR_ORDER GRB
#define NUM_LEDS    23 * 2
CRGB leds[NUM_LEDS];

#define BRIGHTNESS          60  // keep low to save energy
#define FRAMES_PER_SECOND  120
#define MAX_CURRENT       400  // mA, sized to keep USB-only runs inside the 500 mA port limit

// LCD Display
LiquidCrystal_I2C lcd(LCD_I2C_ADDR,16,2);  // 16 chars and 2 line display

// Digits Displays
TM1637TinyDisplay  clockDisplay(PIN_CLK_4DIG, PIN_DIO_4DIG);  // 4 digits with colon
TM1637TinyDisplay6 bigDisplay(PIN_CLK_6DIG, PIN_DIO_6DIG);    // 6 digits

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
    FastLED.clear(true);  // true = push the black frame to the strip right away
    clockDisplay.clear();
    bigDisplay.clear();
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

void initDigitDisplay() {
  clockDisplay.begin();
  bigDisplay.begin();
  clockDisplay.setBrightness(3);
  bigDisplay.setBrightness(3);
}

void loopDigitDisplay() {
  static long n = 0;
  static uint32_t lastUpdate = 0;

  // non-blocking: delay() here would stall the LED animation loop too
  if (millis() - lastUpdate < 100) return;
  lastUpdate = millis();

  // 12:34 with the colon on
  clockDisplay.showNumberDec(1234, 0b01000000, true);

  bigDisplay.showNumber(n++);
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
  // the library gives power at full brightness (assuming 5V), so scale it by our brightness
  uint32_t mA = calculate_unscaled_power_mW(leds, NUM_LEDS) / 5 * BRIGHTNESS / 256;
  // above the cap FastLED dims to exactly MAX_CURRENT, so mark the value instead of hiding it
  bool capped = mA > MAX_CURRENT;
  if (capped) mA = MAX_CURRENT;

  // padded to 16 chars so it overwrites the previous text without clearing the row
  char line[17];
  snprintf(line, sizeof(line), "FPS%4u %c%4umA ", fps, capped ? '>' : ' ', (unsigned)mA);
  lcd.setCursor(0, 1);
  lcd.print(line);

  frames = 0;
  lastUpdate = millis();
}

void setup() {
  initLcd();

  initDigitDisplay();

  pinMode(PIN_BTN, INPUT_PULLUP);

  delay(3000); // 3 second delay for recovery
  
  // tell FastLED about the LED strip configuration
  FastLED.addLeds<LED_TYPE,PIN_LED_DATA,COLOR_ORDER>(leds, NUM_LEDS).setCorrection(TypicalLEDStrip);

  // set master brightness control
  FastLED.setBrightness(BRIGHTNESS);
  // limit current to save energy for a green planet
  FastLED.setMaxPowerInVoltsAndMilliamps(5, MAX_CURRENT);

  applyDemoMode();
}

// Forward declarations for pattern functions
void rainbow();
void rainbowWithGlitter();
void confetti();
void sinelon();
void juggle();
void bpm();
void nextPattern();
void addGlitter(fract8 chanceOfGlitter);

// List of patterns to cycle through.  Each is defined as a separate function below.
typedef void (*SimplePatternList[])();
SimplePatternList gPatterns = { rainbow, rainbowWithGlitter, confetti, sinelon, juggle, bpm };

uint8_t gCurrentPatternNumber = 0; // Index number of which pattern is current
uint8_t gHue = 0; // rotating "base color" used by many of the patterns
  
void loop()
{
  loopButton();

  // Off: strips and displays were blanked once in applyDemoMode(), so just idle
  if (!demoOn) return;

  // Call the current pattern function once, updating the 'leds' array
  gPatterns[gCurrentPatternNumber]();

  // send the 'leds' array out to the actual LED strip
  FastLED.show();  
  // insert a delay to keep the framerate modest
  FastLED.delay(1000/FRAMES_PER_SECOND); 

  // do some periodic updates
  EVERY_N_MILLISECONDS( 20 ) { gHue++; } // slowly cycle the "base color" through the rainbow
  EVERY_N_SECONDS( 10 ) { nextPattern(); } // change patterns periodically


  loopDigitDisplay();
  loopStats();
}

#define ARRAY_SIZE(A) (sizeof(A) / sizeof((A)[0]))

void nextPattern()
{
  // add one to the current pattern number, and wrap around at the end
  gCurrentPatternNumber = (gCurrentPatternNumber + 1) % ARRAY_SIZE( gPatterns);
}

void rainbow() 
{
  // FastLED's built-in rainbow generator
  fill_rainbow( leds, NUM_LEDS, gHue, 7);
}

void rainbowWithGlitter() 
{
  // built-in FastLED rainbow, plus some random sparkly glitter
  rainbow();
  addGlitter(80);
}

void addGlitter( fract8 chanceOfGlitter) 
{
  if( random8() < chanceOfGlitter) {
    leds[ random16(NUM_LEDS) ] += CRGB::White;
  }
}

void confetti() 
{
  // random colored speckles that blink in and fade smoothly
  fadeToBlackBy( leds, NUM_LEDS, 10);
  int pos = random16(NUM_LEDS);
  leds[pos] += CHSV( gHue + random8(64), 200, 255);
}

void sinelon()
{
  // a colored dot sweeping back and forth, with fading trails
  fadeToBlackBy( leds, NUM_LEDS, 20);
  int pos = beatsin16( 13, 0, NUM_LEDS-1 );
  leds[pos] += CHSV( gHue, 255, 192);
}

void bpm()
{
  // colored stripes pulsing at a defined Beats-Per-Minute (BPM)
  uint8_t BeatsPerMinute = 62;
  CRGBPalette16 palette = PartyColors_p;
  uint8_t beat = beatsin8( BeatsPerMinute, 64, 255);
  for( int i = 0; i < NUM_LEDS; i++) { //9948
    leds[i] = ColorFromPalette(palette, gHue+(i*2), beat-gHue+(i*10));
  }
}

void juggle() {
  // eight colored dots, weaving in and out of sync with each other
  fadeToBlackBy( leds, NUM_LEDS, 20);
  uint8_t dothue = 0;
  for( int i = 0; i < 8; i++) {
    leds[beatsin16( i+7, 0, NUM_LEDS-1 )] |= CHSV(dothue, 200, 255);
    dothue += 32;
  }
}
