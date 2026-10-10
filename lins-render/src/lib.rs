//! Lins drawn by Rust.
//!
//! One scene: a node in the middle as a card that holds its whole name, what points at it in
//! columns on the left, what it points at on the right, links between them. The same scene is
//! drawn by the browser in `lins-web/src/routes/trial`, by the same rules, so the two can be
//! looked at side by side.
//!
//! - [`scene`]: what a scene is and where its parts go. No drawing, no web.
//! - [`view`]: draws a placed scene with egui, and pans and zooms it.
//! - `web`: what a web page calls, when built for the web.

pub mod scene;
pub mod view;

#[cfg(target_arch = "wasm32")]
mod web;
