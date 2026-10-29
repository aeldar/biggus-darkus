/// @file    app_mode.h
/// @brief   Global mode switch shared by the button and serial commands.

#pragma once

#include <stdint.h>

enum class Mode : uint8_t { Off, Demo, Active };

void initMode();          // start in DEMO and apply it
Mode currentMode();
void setMode(Mode mode);
void nextMode();          // button cycle: Demo -> Active -> Off -> Demo
