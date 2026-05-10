// Copyright (c) 2026 Vu. All rights reserved.
// Proprietary source. See OWNERSHIP.md at the repository root.

fn main() {
    let ownership = std::path::Path::new("../OWNERSHIP.md");
    if !ownership.exists() {
        panic!("Missing required OWNERSHIP.md file. The application cannot be built without the ownership notice.");
    }

    let content =
        std::fs::read_to_string(ownership).expect("Unable to read required OWNERSHIP.md file.");
    if !content.contains("AGENTS_CONTROL_CENTER_OWNER_FILE_V1") {
        panic!("OWNERSHIP.md is missing the required ownership marker.");
    }

    println!("cargo:rerun-if-changed=../OWNERSHIP.md");
    tauri_build::build()
}
