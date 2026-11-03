# Serial commands

The board listens on its USB serial port at **9600 baud, 8N1**. Send one command per
line, terminated with `\n`. `\r` is ignored. In the Arduino IDE serial monitor, set
9600 baud and line ending `Newline`.

- Device, action and keyword arguments are case-insensitive.
- LCD text keeps its case.
- Maximum line length is 64 characters; longer lines get `ERR LINE`.
- Successful commands answer `OK`.

## Modes

| Command | Effect |
|---|---|
| `MODE DEMO` | LED patterns, LCD stats, fixed digit demo values |
| `MODE ACTIVE` | Accept hardware commands |
| `MODE OFF` | Blank the LED strip, LCD and digit displays |

`MODE` works in every mode. All other commands are ignored unless the board is in
`ACTIVE`; they answer `ERR MODE`.

The button on pin `A0` cycles `DEMO → ACTIVE → OFF → DEMO`.

## 4-digit display

The display has three independent modes. Switching away keeps the other modes'
state; switching back resumes the selected mode.

| Command | Effect |
|---|---|
| `4DIGIT TIME` | Switch to the clock, resuming its saved time |
| `4DIGIT TIME 12:34` | Set the clock to 12:34 and run |
| `4DIGIT STARTWATCH` | Switch to the startwatch, resuming it |
| `4DIGIT STARTWATCH 90` | Start the startwatch at 1:30 |
| `4DIGIT STARTWATCH PAUSE` | Switch to the startwatch and pause it |
| `4DIGIT TIMER` | Switch to the timer, resuming it |
| `4DIGIT TIMER 300` | Set the timer to 5:00 and run |
| `4DIGIT TIMER PAUSE` | Switch to the timer and pause it |

- `TIME` takes `HH:MM` in 24-hour form.
- `STARTWATCH` and `TIMER` take seconds.
- A command without a value switches to that mode and resumes it.
- `PAUSE` freezes the mode at its current value. Selecting it again resumes.
- The clock always shows `hh:mm`. Startwatch and timer show `mm:ss` below one hour,
  then `hh:mm`.

## 6-digit display

| Command | Effect |
|---|---|
| `6DIGIT SET 12345` | Show 12345 and stop changing |
| `6DIGIT RESET` | Show 0 and stop changing |
| `6DIGIT COUNTUP 100` | Count up from 100, default step 100 ms |
| `6DIGIT COUNTUP 100 1000` | Count up from 100, one step per second |
| `6DIGIT COUNTDOWN 100` | Count down from 100, default step 100 ms |
| `6DIGIT COUNTDOWN 100 1000` | Count down from 100, one step per second |

The optional last argument is the step period in milliseconds. Count-down stops at
0. A hidden counter keeps running.

## LCD

| Command | Effect |
|---|---|
| `LCD ROW 1 Hello` | Write `Hello` on row 1 |
| `LCD ROW 2 Ready` | Write `Ready` on row 2 |
| `LCD ROW 1` | Clear row 1 |
| `LCD CLEAR` | Clear both rows |

Text is truncated to 16 characters. Row 2 is used by the FPS/current stats in
`DEMO` mode, so text written in `ACTIVE` disappears when you switch back to `DEMO`.

## LED strip

Ranges are `[start, end)`, with `start` and `end` in `0..46`. `0 23` is the first
23 pixels. Every LED keeps its own state: off, solid, blink or pulse.

| Command | Effect |
|---|---|
| `LED_STRIP RANGE 0 23 COLOR FF0000 ANIM SOLID` | Solid red on the first 23 pixels |
| `LED_STRIP RANGE 0 23 COLOR 00FF00 ANIM BLINK 500` | Green blink, 500 ms period |
| `LED_STRIP RANGE 0 23 COLOR 0000FF ANIM PULSE 1000` | Blue pulse, 1000 ms period |
| `LED_STRIP RANGE 0 23 CLEAR` | Turn off the LEDs in that range |
| `LED_STRIP CLEAR` | Turn off all LEDs |

- Colour is `RRGGBB` in hex.
- `ANIM` is `SOLID`, `BLINK` or `PULSE`.
- The optional period is in milliseconds; defaults are 500 ms for blink and 1000 ms
  for pulse.
- LED states are visible in `ACTIVE` on an otherwise blank strip. They are hidden in
  `DEMO` and `OFF` but preserved.
- A new command replaces the state of every LED in its range, so no old animation
  survives there. LEDs outside the range keep their state.

## Replies

| Reply | Meaning |
|---|---|
| `OK` | Command applied |
| `ERR MODE` | Device command sent outside `ACTIVE` |
| `ERR UNKNOWN` | Unknown device |
| `ERR ARGS` | Missing or malformed argument |
| `ERR RANGE` | Number or range out of bounds |
| `ERR LINE` | Line longer than 64 characters |

## Examples

```text
MODE ACTIVE
OK
4DIGIT TIMER 300
OK
6DIGIT SET 42
OK
LCD ROW 1 Biggus
OK
LED_STRIP RANGE 0 23 COLOR FF0000 ANIM SOLID
OK
LED_STRIP RANGE 23 46 COLOR 0000FF ANIM PULSE 1000
OK
MODE DEMO
OK
```
