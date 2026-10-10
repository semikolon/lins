//! Says whether the page's own fonts have been unpacked for this crate.
//!
//! scripts/build-render.sh writes them to assets/fonts/, which git ignores. A build without them
//! still compiles and draws with egui's built-in face, so `cargo check --workspace` on a fresh
//! clone does not fail on a missing file.

use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(lins_fonts)");
    let fonts = ["assets/fonts/Manrope-Medium.ttf", "assets/fonts/Manrope-SemiBold.ttf"];
    for font in fonts {
        println!("cargo::rerun-if-changed={font}");
    }
    if fonts.iter().all(|font| Path::new(font).is_file()) {
        println!("cargo::rustc-cfg=lins_fonts");
    } else {
        println!("cargo::warning=lins-render: no fonts in assets/fonts, drawing with egui's built-in face. Run scripts/build-render.sh.");
    }
}
