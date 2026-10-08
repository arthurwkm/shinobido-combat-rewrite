//! The action table is generated from the player's own copy of the game and is not committed.

use std::path::Path;

const GENERATED: &str = "src/sim/extracted.rs";

fn main() {
    println!("cargo:rerun-if-changed={GENERATED}");
    if !Path::new(GENERATED).exists() {
        eprintln!("\n{GENERATED} is missing. It is generated from the game disc; run:\n");
        eprintln!("    .venv/bin/python tools/volume.py extract 'motion\\'");
        eprintln!("    .venv/bin/python tools/extract.py\n");
        std::process::exit(1);
    }
}
