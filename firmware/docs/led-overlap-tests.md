# LED range overlap tests

Manual tests for overlapping `LED_STRIP RANGE` commands. Every LED keeps its own
state, so a new range overwrites the state of each LED it covers.

Open the serial monitor at 9600 baud with line ending `Newline`, then send the
commands one at a time. Each command answers `OK`.

```text
MODE ACTIVE
```

## 1. Overlap on one side

```text
LED_STRIP CLEAR
LED_STRIP RANGE 0 20 COLOR FF0000 ANIM SOLID
LED_STRIP RANGE 10 30 COLOR 0000FF ANIM SOLID
```

Expect: LEDs 0-9 red, 10-29 blue. No red or purple mixed into the blue part.

## 2. New range inside an old one

```text
LED_STRIP CLEAR
LED_STRIP RANGE 0 40 COLOR FF0000 ANIM SOLID
LED_STRIP RANGE 15 25 COLOR 00FF00 ANIM BLINK 500
```

Expect: 0-14 and 25-39 solid red, 15-24 blinking green. While the green is off,
15-24 must be dark, not red. This was the original bug.

## 3. New range covers an old one

```text
LED_STRIP CLEAR
LED_STRIP RANGE 5 10 COLOR FF0000 ANIM PULSE 1000
LED_STRIP RANGE 0 20 COLOR 00FF00 ANIM SOLID
```

Expect: 0-19 solid green, no red pulse.

## 4. Clear part of a range

```text
LED_STRIP CLEAR
LED_STRIP RANGE 0 20 COLOR FF0000 ANIM SOLID
LED_STRIP RANGE 5 10 CLEAR
```

Expect: 0-4 and 10-19 red, 5-9 dark.
