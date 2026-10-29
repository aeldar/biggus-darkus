# Hardware

Pin assignments live in [`BiggusDarkus/pins.h`](../BiggusDarkus/pins.h) — that file is the
source of truth. Update it first, then this page.

## Board

Arduino Nano — 5 V logic, so the strip and both TM1637 modules run at the same
level and need no shifters. Its I²C bus is fixed to A4 (SDA) and A5 (SCL); the
firmware never names those pins because `Wire` selects them.

## Signals

| Signal | MCU pin | Connects to | Notes |
|---|---|---|---|
| LED data | `4` (`PIN_LED_DATA`) | WS2811 strip data in | Single-wire, no clock |
| Mode button | `A0` (`PIN_BTN`) | Button to ground | `INPUT_PULLUP`, 50 ms debounce; press cycles Demo → Active → Off |
| 4-digit display CLK | `5` (`PIN_CLK_4DIG`) | 4-digit TM1637 | |
| 4-digit display DIO | `6` (`PIN_DIO_4DIG`) | 4-digit TM1637 | |
| 6-digit display CLK | `7` (`PIN_CLK_6DIG`) | 6-digit TM1637 | |
| 6-digit display DIO | `8` (`PIN_DIO_6DIG`) | 6-digit TM1637 | |
| LCD SDA | `A4` | 16×2 I²C backpack | Address `0x27` (`LCD_I2C_ADDR`) |
| LCD SCL | `A5` | 16×2 I²C backpack | Same bus as above |

## LED strip

- Type `WS2811`, colour order `GRB`, 46 pixels (two runs of 23).
- Powered from 5 V; firmware caps strip draw at 400 mA via
  `setMaxPowerInVoltsAndMilliamps(5, 400)`. The cap is a per-frame dimmer, not a
  fuse: frames estimated above it are scaled down until they fit, so the brightest
  patterns run visibly dimmer than sparse ones.
- Master brightness `60` of 255 — deliberately low to save energy.
- Demo mode runs the pattern rotation; active mode shows only command overlays on a
  blank strip.
- The strip needs its own 5 V supply with a shared ground for full output; the Nano
  only drives the data line.

## Power supply

Deliberately not part of the design — any 5 V USB supply works, currently an old
2 A phone charger. Because the weakest source you might plug in is a plain USB port
(500 mA), `kMaxCurrentMa` is set low enough to be safe everywhere:

| Source | Budget | Notes |
|---|---|---|
| USB port via the Nano | 500 mA | `kMaxCurrentMa` 400 + ~70 mA board and displays ≈ 470 mA |
| Any 5 V charger, 1 A | ~930 mA usable | cap leaves most of it unused |
| Current 2 A charger | ~1930 mA usable | same |

If a dedicated supply ever becomes part of the build, raise `kMaxCurrentMa` to its
rating minus ~70 mA of overhead — or keep 400 and accept the dimming, since
brightness is already sufficient.

## Displays

- **LCD** — 16×2 character module on an I²C backpack, address `0x27`. Row 1 shows the
  global mode (`Demo ON`, `Mode ACTIVE`, or blank when off); row 2 shows measured FPS
  and estimated strip current in demo mode, or command text in active mode. A `>`
  before the number means the estimate went past the current cap and the strip is
  being dimmed to fit; without it, the reading is the true expected draw.
- **4-digit TM1637** — three independent time modes: clock (`hh:mm`, wraps at 24:00),
  startwatch (counts up), and timer (counts down, stops at `00:00`). Startwatch and
  timer show `mm:ss` below one hour, then `hh:mm`. Demo mode shows a fixed `12:34`.
- **6-digit TM1637** — static value, count up, or count down. Demo mode shows a fixed
  `123456`.
- Both TM1637 displays run at brightness level 3.

## Serial

The USB serial port runs the command console at **9600 baud**. Commands are
newline-terminated; see [`commands.md`](commands.md). 9600 is deliberate: the LED
frame functions block for about 8 ms, and at 115200 the 64-byte receive buffer can
overflow during a burst.

## Libraries

| Library | Used for |
|---|---|
| `FastLED` | Strip driving and the effect patterns |
| `LiquidCrystal_I2C` (YWROBOT, 1.1) | LCD over I²C |
| `TM1637TinyDisplay`, `TM1637TinyDisplay6` | Digit displays |
| `Wire` | I²C bus |

## Not yet documented

Unknown at the moment — fill these in when the build is fixed:

- Fusing and whether the strip has power injection.
- Connectors, wire colours, and cable lengths.
- Any series resistor or level shifter on the LED data line.
- Enclosure and physical mounting.
- Photo or schematic of the assembled unit.
