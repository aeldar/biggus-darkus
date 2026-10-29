# Architecture

The firmware is a set of small modules around one global mode switch. The sketch only
initialises the modules and runs the loop; behaviour lives in the modules.

## Modes

There are three global modes, defined in `app_mode.h`:

| Mode | LED strip | LCD | Digit displays | Device commands |
|---|---|---|---|---|
| `OFF` | blank | blank | blank | ignored |
| `DEMO` | pattern rotation | `Demo ON` + FPS/current | fixed demo values | ignored |
| `ACTIVE` | command overlays only | command text | command-driven | executed |

The board starts in `DEMO`. `MODE` is always accepted. Every other command is
accepted only in `ACTIVE`; in `DEMO` and `OFF` it is ignored and answered with
`ERR MODE`.

The button cycles `DEMO → ACTIVE → OFF → DEMO`, so the board can always be taken out
of `ACTIVE` without a serial connection.

Mode transitions live in `app_mode.cpp`:

- Entering `OFF` blanks the LCD, both digit displays and the strip. Overlay state is
  kept.
- Entering `DEMO` clears the strip for a clean start, writes the fixed digit demo
  values and shows `Demo ON`. Overlays are hidden, not cleared.
- Entering `ACTIVE` shows `Mode ACTIVE`; the demo pattern is hidden and only the
  command overlays are shown.

The digit displays and LED overlays keep their own active state while demo values
are shown, so switching back to `ACTIVE` restores them.

## Modules

| File | Responsibility |
|---|---|
| `BiggusDarkus.ino` | Initialisation and the mode switch in `loop()` |
| `app_mode.*` | Global mode state, transitions, button cycle |
| `serial_commands.*` | Serial line reader, parser and command routing |
| `led_strip.*` | FastLED setup, demo patterns, command overlays, power estimate |
| `four_digit_display.*` | 4-digit clock, startwatch and timer |
| `counter_display.*` | 6-digit static, count-up and count-down modes |
| `status_lcd.*` | 16×2 LCD text and demo stats |
| `button.*` | Debounced mode button |
| `pins.h` | Pin assignments |

## Loop

```cpp
void loop() {
  loopSerialCommands();

  if (buttonPressed()) nextMode();

  switch (currentMode()) {
    case Mode::Off:
      break;

    case Mode::Demo:
      showLedDemoFrame();
      loopStats();
      break;

    case Mode::Active:
      showActiveLedFrame();
      loopFourDigitDisplay();
      loopCounterDisplay();
      break;
  }
}
```

The digit demo values are written once on entering `DEMO`; nothing needs to run for
them every frame. The LCD stats still refresh in `DEMO`.

## 4-digit display

The display has three independent modes:

| Mode | Behaviour |
|---|---|
| `TIME` | Clock, `hh:mm`, wraps at 24:00 |
| `STARTWATCH` | Counts up, `mm:ss` below one hour, then `hh:mm` |
| `TIMER` | Counts down, `mm:ss` below one hour, then `hh:mm`, stops at `00:00` |

Each mode stores only its base value and the moment it started:

```cpp
struct ModeState {
  uint32_t baseSeconds;
  uint32_t startedAtMs;
  bool running;
};
```

The current value is computed from `millis()` on demand. No ticks are stored. A
hidden mode keeps running, so a timer still counts down while the clock is shown.

`PAUSE` stores the current value in `baseSeconds` and clears `running`. Selecting a
paused mode resumes it from that value. Starting a mode with a new value resets only
that mode.

Demo mode ignores all three states and writes a fixed `12:34`.

## 6-digit display

Three modes:

| Mode | Behaviour |
|---|---|
| Static | Shows one value, no change |
| Count up | Increases from a start value at a fixed period |
| Count down | Decreases from a start value, stops at 0 |

Like the 4-digit display, the value is derived from the start time and period, so a
hidden counter keeps advancing. Demo mode writes a fixed `123456` and leaves the
active value untouched.

## LED strip

- `DEMO` runs the FastLED DemoReel100 pattern rotation without overlays.
- `ACTIVE` shows a blank strip with up to four command overlays on top, then pushes
  the frame. The demo pattern is not drawn.
- An overlay is a range `[start, end)`, a colour, an animation (`SOLID`, `BLINK`,
  `PULSE`) and a period.
- Overlays survive mode changes. `OFF` blanks the strip but keeps them; `DEMO` hides
  them; `ACTIVE` shows them again. `LED_STRIP CLEAR` removes them.

The strip is still capped at 400 mA and brightness 60.

## Serial input

`serial_commands.cpp` reads characters without blocking:

1. Assemble a line into a 64-byte buffer, ignoring `\r`, finishing on `\n`.
2. Split the line into device, action and argument text.
3. Uppercase device and action, then route.

Device commands run only in `ACTIVE`. `MODE` is handled first and always works.

The console runs at **9600 baud**. The LED frame functions block for about 8 ms; at
115200 that is enough to overrun the Nano's 64-byte serial receive buffer during a
burst. 9600 leaves plenty of headroom.

## Memory

The firmware uses about 73% of flash and 58% of SRAM on the ATmega328. The main RAM
costs are the Serial library buffers, the 64-byte command line and the four overlay
slots. There is no dynamic allocation.
