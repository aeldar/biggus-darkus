/// @file    serial_commands.cpp
/// @brief   Reads newline-terminated commands and routes them to the hardware.
/// @note    Device commands run only in ACTIVE mode; MODE always works.

#include "serial_commands.h"

#include <Arduino.h>
#include <stdlib.h>
#include <string.h>

#include "app_mode.h"
#include "counter_display.h"
#include "four_digit_display.h"
#include "led_strip.h"
#include "status_lcd.h"

namespace {

constexpr uint32_t kBaud = 9600;
constexpr size_t kLineMax = 64;
constexpr long kCounterMax = 999999;
constexpr long kTimeMaxSeconds = 359999;  // 99:59:59

char line[kLineMax + 1];
char emptyArg[1] = {0};
size_t length = 0;
bool overflow = false;

void reply(const __FlashStringHelper* text) {
  Serial.println(text);
}

void uppercase(char* text) {
  for (; *text != '\0'; text++) {
    if (*text >= 'a' && *text <= 'z') *text = *text - 'a' + 'A';
  }
}

bool parseLong(const char* text, long& value) {
  char* end = nullptr;
  long parsed = strtol(text, &end, 10);
  if (end == text || *end != '\0') return false;
  value = parsed;
  return true;
}

// HH:MM -> seconds of day
bool parseClock(const char* text, uint32_t& seconds) {
  char* end = nullptr;
  long hours = strtol(text, &end, 10);
  if (end == text || *end != ':') return false;

  char* minuteEnd = nullptr;
  long minutes = strtol(end + 1, &minuteEnd, 10);
  if (minuteEnd == end + 1 || *minuteEnd != '\0') return false;
  if (hours < 0 || hours > 23 || minutes < 0 || minutes > 59) return false;

  seconds = (uint32_t)(hours * 3600 + minutes * 60);
  return true;
}

// RRGGBB -> three bytes
bool parseColor(const char* text, uint8_t& red, uint8_t& green, uint8_t& blue) {
  if (strlen(text) != 6) return false;

  char* end = nullptr;
  unsigned long value = strtoul(text, &end, 16);
  if (end == text || *end != '\0') return false;

  red = (value >> 16) & 0xFF;
  green = (value >> 8) & 0xFF;
  blue = value & 0xFF;
  return true;
}

bool parseAnimation(const char* text, LedAnimation& animation) {
  if (strcmp(text, "SOLID") == 0) {
    animation = LedAnimation::Solid;
    return true;
  }
  if (strcmp(text, "BLINK") == 0) {
    animation = LedAnimation::Blink;
    return true;
  }
  if (strcmp(text, "PULSE") == 0) {
    animation = LedAnimation::Pulse;
    return true;
  }
  return false;
}

void handleMode(char* action) {
  if (!action) {
    reply(F("ERR ARGS"));
    return;
  }

  if (strcmp(action, "DEMO") == 0) {
    setMode(Mode::Demo);
  } else if (strcmp(action, "ACTIVE") == 0) {
    setMode(Mode::Active);
  } else if (strcmp(action, "OFF") == 0) {
    setMode(Mode::Off);
  } else {
    reply(F("ERR ARGS"));
    return;
  }
  reply(F("OK"));
}

void handle4Digit(char* action, char* args) {
  if (!action) {
    reply(F("ERR ARGS"));
    return;
  }

  TimeMode mode;
  bool clock = false;
  if (strcmp(action, "TIME") == 0) {
    mode = TimeMode::Time;
    clock = true;
  } else if (strcmp(action, "STARTWATCH") == 0) {
    mode = TimeMode::Startwatch;
  } else if (strcmp(action, "TIMER") == 0) {
    mode = TimeMode::Timer;
  } else {
    reply(F("ERR ARGS"));
    return;
  }

  char* save = nullptr;
  char* arg = strtok_r(args, " ", &save);
  if (!arg) {
    selectTimeMode(mode);
    reply(F("OK"));
    return;
  }

  uppercase(arg);

  if (!clock && strcmp(arg, "PAUSE") == 0) {
    pauseTimeMode(mode);
    reply(F("OK"));
    return;
  }

  uint32_t value;
  if (clock) {
    if (!parseClock(arg, value)) {
      reply(F("ERR ARGS"));
      return;
    }
  } else {
    long seconds;
    if (!parseLong(arg, seconds) || seconds < 0 || seconds > kTimeMaxSeconds) {
      reply(F("ERR RANGE"));
      return;
    }
    value = (uint32_t)seconds;
  }

  startTimeMode(mode, value);
  reply(F("OK"));
}

void handle6Digit(char* action, char* args) {
  if (!action) {
    reply(F("ERR ARGS"));
    return;
  }

  if (strcmp(action, "RESET") == 0) {
    resetCounter();
    reply(F("OK"));
    return;
  }

  char* save = nullptr;
  char* arg = strtok_r(args, " ", &save);
  if (!arg) {
    reply(F("ERR ARGS"));
    return;
  }

  long value;
  if (!parseLong(arg, value) || value < 0 || value > kCounterMax) {
    reply(F("ERR RANGE"));
    return;
  }

  if (strcmp(action, "SET") == 0) {
    setCounter(value);
    reply(F("OK"));
    return;
  }

  uint16_t period = 100;
  char* periodArg = strtok_r(nullptr, " ", &save);
  if (periodArg) {
    long parsed;
    if (!parseLong(periodArg, parsed) || parsed <= 0 || parsed > 65535L) {
      reply(F("ERR RANGE"));
      return;
    }
    period = (uint16_t)parsed;
  }

  if (strcmp(action, "COUNTUP") == 0) {
    startCounterUp(value, period);
  } else if (strcmp(action, "COUNTDOWN") == 0) {
    startCounterDown(value, period);
  } else {
    reply(F("ERR ARGS"));
    return;
  }
  reply(F("OK"));
}

void handleLcd(char* action, char* args) {
  if (!action) {
    reply(F("ERR ARGS"));
    return;
  }

  if (strcmp(action, "CLEAR") == 0) {
    clearStatusLcd();
    reply(F("OK"));
    return;
  }

  if (strcmp(action, "ROW") != 0) {
    reply(F("ERR ARGS"));
    return;
  }

  char* save = nullptr;
  char* rowArg = strtok_r(args, " ", &save);
  long row;
  if (!rowArg || !parseLong(rowArg, row) || row < 1 || row > 2) {
    reply(F("ERR ARGS"));
    return;
  }

  showLcdRow((uint8_t)row, save);  // rest of the line keeps its case and spaces
  reply(F("OK"));
}

void handleLedStrip(char* action, char* args) {
  if (!action) {
    reply(F("ERR ARGS"));
    return;
  }

  if (strcmp(action, "CLEAR") == 0) {
    clearLedRanges();
    reply(F("OK"));
    return;
  }

  if (strcmp(action, "RANGE") != 0) {
    reply(F("ERR ARGS"));
    return;
  }

  char* save = nullptr;
  char* startArg = strtok_r(args, " ", &save);
  if (!startArg) {
    reply(F("ERR RANGE"));
    return;
  }
  char* endArg = strtok_r(nullptr, " ", &save);
  long start;
  long end;
  if (!endArg || !parseLong(startArg, start) || !parseLong(endArg, end) ||
      start < 0 || end <= start || end > kLedCount) {
    reply(F("ERR RANGE"));
    return;
  }

  char* keyword = strtok_r(nullptr, " ", &save);
  if (!keyword) {
    reply(F("ERR ARGS"));
    return;
  }
  uppercase(keyword);

  if (strcmp(keyword, "CLEAR") == 0) {
    clearLedRange((uint8_t)start, (uint8_t)end);
    reply(F("OK"));
    return;
  }

  if (strcmp(keyword, "COLOR") != 0) {
    reply(F("ERR ARGS"));
    return;
  }

  char* colorArg = strtok_r(nullptr, " ", &save);
  uint8_t red;
  uint8_t green;
  uint8_t blue;
  if (!colorArg || !parseColor(colorArg, red, green, blue)) {
    reply(F("ERR ARGS"));
    return;
  }

  keyword = strtok_r(nullptr, " ", &save);
  if (!keyword) {
    reply(F("ERR ARGS"));
    return;
  }
  uppercase(keyword);
  if (strcmp(keyword, "ANIM") != 0) {
    reply(F("ERR ARGS"));
    return;
  }

  char* animArg = strtok_r(nullptr, " ", &save);
  if (!animArg) {
    reply(F("ERR ARGS"));
    return;
  }
  uppercase(animArg);

  LedAnimation animation;
  if (!parseAnimation(animArg, animation)) {
    reply(F("ERR ARGS"));
    return;
  }

  uint16_t period = 0;
  if (animation == LedAnimation::Blink) period = 500;
  if (animation == LedAnimation::Pulse) period = 1000;

  char* periodArg = strtok_r(nullptr, " ", &save);
  if (periodArg) {
    long parsed;
    if (!parseLong(periodArg, parsed) || parsed <= 0 || parsed > 65535L) {
      reply(F("ERR RANGE"));
      return;
    }
    period = (uint16_t)parsed;
  }

  if (!setLedRange((uint8_t)start, (uint8_t)end, red, green, blue, animation, period)) {
    reply(F("ERR FULL"));
    return;
  }
  reply(F("OK"));
}

void handleLine(char* text) {
  char* save = nullptr;
  char* device = strtok_r(text, " ", &save);
  if (!device) return;

  char* action = strtok_r(nullptr, " ", &save);
  char* args = save ? save : emptyArg;

  uppercase(device);
  if (action) uppercase(action);

  if (strcmp(device, "MODE") == 0) {
    handleMode(action);
    return;
  }

  if (currentMode() != Mode::Active) {
    reply(F("ERR MODE"));
    return;
  }

  if (strcmp(device, "4DIGIT") == 0) {
    handle4Digit(action, args);
  } else if (strcmp(device, "6DIGIT") == 0) {
    handle6Digit(action, args);
  } else if (strcmp(device, "LCD") == 0) {
    handleLcd(action, args);
  } else if (strcmp(device, "LED_STRIP") == 0) {
    handleLedStrip(action, args);
  } else {
    reply(F("ERR UNKNOWN"));
  }
}

}  // namespace

void initSerialCommands() {
  Serial.begin(kBaud);
  length = 0;
  overflow = false;
}

void loopSerialCommands() {
  while (Serial.available() > 0) {
    char c = Serial.read();

    if (c == '\r') continue;

    if (c == '\n') {
      if (overflow) {
        reply(F("ERR LINE"));
      } else {
        line[length] = '\0';
        handleLine(line);
      }
      length = 0;
      overflow = false;
      continue;
    }

    if (length < kLineMax) {
      line[length++] = c;
    } else {
      overflow = true;
    }
  }
}
