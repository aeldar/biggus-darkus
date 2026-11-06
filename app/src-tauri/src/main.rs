//! Desktop entry point; all logic lives in the library.

// Keeps Windows from opening a console next to the window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bd_app_lib::run();
}
