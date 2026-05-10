// Copyright (c) 2026 Vu. All rights reserved.
// Proprietary source. See OWNERSHIP.md at the repository root.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}
