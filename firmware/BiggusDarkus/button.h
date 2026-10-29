/// @file    button.h
/// @brief   Debounced demo mode button.

#pragma once

void initButton();

// True once per debounced press (HIGH -> LOW edge).
bool buttonPressed();
