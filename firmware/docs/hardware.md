# Hardware

Pin assignments live in [`BiggusDarkus/pins.h`](../BiggusDarkus/pins.h) — that file is the
source of truth. Update it first, then this page.

## Signals

| Signal | MCU pin | Connects to | Notes |
|---|---|---|---|
| LED data | `4` (`PIN_LED_DATA`) | WS2811 strip data in | Single-wire, no clock |
| Mode button | `A0` (`PIN_BTN`) | Button to ground | `INPUT_PULLUP`, 50 ms debounce; press toggles demo mode |
| Clock display CLK | `5` (`PIN_CLK_4DIG`) | 4-digit TM1637 | |
| Clock display DIO | `6` (`PIN_DIO_4DIG`) | 4-digit TM1637 | |
| Counter display CLK | `7` (`PIN_CLK_6DIG`) | 6-digit TM1637 | |
| Counter display DIO | `8` (`PIN_DIO_6DIG`) | 6-digit TM1637 | |
| LCD SDA/SCL | board default | 16×2 I²C backpack | Address `0x27` (`LCD_I2C_ADDR`) |

## LED strip

- Type `WS2811`, colour order `GRB`, 46 pixels (two runs of 23).
- Powered from 5 V; firmware caps draw at 1000 mA via
  `setMaxPowerInVoltsAndMilliamps(5, 1000)`, so FastLED dims the strip rather than
  overdrawing the supply.
- Master brightness `60` of 255 — deliberately low to save energy.

## Displays

- **LCD** — 16×2 character module on an I²C backpack, address `0x27`. Row 1 shows
  `Demo ON` / `Demo OFF`; row 2 shows measured FPS and estimated strip current in mA.
- **4-digit TM1637** — shows a fixed `12:34` with the colon lit.
- **6-digit TM1637** — shows a running counter.
- Both TM1637 displays run at brightness level 3 and refresh at 10 Hz.

## Libraries

| Library | Used for |
|---|---|
| `FastLED` | Strip driving and the effect patterns |
| `LiquidCrystal_I2C` (YWROBOT, 1.1) | LCD over I²C |
| `TM1637TinyDisplay`, `TM1637TinyDisplay6` | Digit displays |
| `Wire` | I²C bus |

## Not yet documented

Unknown at the moment — fill these in when the build is fixed:

- Exact MCU board and how its I²C pins are broken out.
- Power supply rating, fusing, and whether the strip has power injection.
- Connectors, wire colours, and cable lengths.
- Any series resistor or level shifter on the LED data line.
- Enclosure and physical mounting.
- Photo or schematic of the assembled unit.
