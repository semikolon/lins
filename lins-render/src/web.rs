//! What a web page calls.
//!
//! The page makes one [`WebHandle`], gives it a canvas, and hands it a scene as JSON. Everything
//! the page says goes into a small shared note that the drawing reads at its next frame, so the
//! page never reaches into a frame that is being drawn.
//!
//! A canvas holds no text a page can read. So the handle also gives back what it is showing as
//! data ([`WebHandle::placed`]): every box with its text and its place. A test, a reader for the
//! blind, or the page's own search can use that where they cannot use the picture.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::scene::{self, Heights, Scene};
use crate::view::{install_fonts, SceneView, TextMode};

#[derive(Default)]
struct Shared {
    ctx: Option<egui::Context>,

    // From the page, taken by the next frame.
    scene: Option<Scene>,
    view: Option<[f32; 3]>,
    fit: bool,
    text_mode: Option<u8>,
    on_pick: Option<js_sys::Function>,

    // From the last frame, for the page.
    placed: String,
    placed_count: u64,
    now: [f32; 3],
    text_zoom: f32,
    frames: u64,
    frame_ms: f32,
    cut: usize,
    own_fonts: bool,
}

impl Shared {
    fn wake(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
}

struct App {
    shared: Rc<RefCell<Shared>>,
    view: SceneView,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, shared: Rc<RefCell<Shared>>) -> Self {
        let own_fonts = install_fonts(&cc.egui_ctx);
        {
            let mut s = shared.borrow_mut();
            s.ctx = Some(cc.egui_ctx.clone());
            s.own_fonts = own_fonts;
        }
        Self { shared, view: SceneView::default() }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        {
            let mut s = self.shared.borrow_mut();
            if let Some(scene) = s.scene.take() {
                self.view.set_scene(scene);
            }
            if let Some(mode) = s.text_mode.take() {
                self.view.set_text_mode(TextMode::from_number(mode));
            }
            if std::mem::take(&mut s.fit) {
                self.view.fit_next_frame();
            }
            if let Some([x, y, zoom]) = s.view.take() {
                self.view.set_view(x, y, zoom);
            }
        }

        self.view.ui(ui);

        let picked = {
            let mut s = self.shared.borrow_mut();
            if s.placed_count != self.view.placed() {
                s.placed_count = self.view.placed();
                s.placed = self.view.json().to_owned();
            }
            s.now = self.view.view();
            s.text_zoom = self.view.text_zoom();
            s.frames = self.view.frames();
            s.cut = self.view.cut();
            if let Some(seconds) = frame.info().cpu_usage {
                s.frame_ms = seconds * 1000.0;
            }
            self.view.take_picked().and_then(|id| s.on_pick.clone().map(|call| (call, id)))
        };
        // Called with the note put down: the page may call straight back into the handle.
        if let Some((call, id)) = picked {
            let _ = call.call1(&JsValue::NULL, &JsValue::from_str(&id));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.view.canvas_colour().to_normalized_gamma_f32()
    }
}

/// The page's hold on one Rust-drawn scene.
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
    shared: Rc<RefCell<Shared>>,
}

#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        eframe::WebLogger::init(log::LevelFilter::Warn).ok();
        Self { runner: eframe::WebRunner::new(), shared: Rc::default() }
    }

    /// Start drawing on a canvas. The canvas takes the size its own CSS gives it.
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        let shared = self.shared.clone();
        self.runner
            .start(canvas, eframe::WebOptions::default(), Box::new(move |cc| Ok(Box::new(App::new(cc, shared)))))
            .await
    }

    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// Show this scene, fitted to the canvas. The error says which field of the JSON is wrong.
    pub fn set_scene(&self, json: &str) -> Result<(), JsValue> {
        let scene: Scene = serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut s = self.shared.borrow_mut();
        s.scene = Some(scene);
        s.wake();
        Ok(())
    }

    /// Put the scene's own (0, 0) at `(x, y)` on the canvas, in CSS pixels, at this zoom.
    pub fn set_view(&self, x: f32, y: f32, zoom: f32) {
        let mut s = self.shared.borrow_mut();
        s.view = Some([x, y, zoom]);
        s.wake();
    }

    /// Show the whole scene.
    pub fn fit(&self) {
        let mut s = self.shared.borrow_mut();
        s.fit = true;
        s.wake();
    }

    /// How text is drawn while zoomed: 0 stretched, 1 sharp once the zoom rests, 2 always sharp.
    pub fn set_text_mode(&self, mode: u8) {
        let mut s = self.shared.borrow_mut();
        s.text_mode = Some(mode);
        s.wake();
    }

    /// Call `f(id)` when a card is clicked.
    pub fn on_pick(&self, f: js_sys::Function) {
        self.shared.borrow_mut().on_pick = Some(f);
    }

    /// `[x, y, zoom]` as last drawn.
    pub fn view(&self) -> Vec<f32> {
        self.shared.borrow().now.to_vec()
    }

    /// What is on the canvas, as JSON: every box with its text and place, and the links.
    pub fn placed(&self) -> String {
        self.shared.borrow().placed.clone()
    }

    /// Counts for the page, as JSON: frames drawn, the last frame's time on the processor in
    /// milliseconds, the zoom the text is set at, names cut short, and whether this build
    /// carries the page's own fonts.
    pub fn stats(&self) -> String {
        let s = self.shared.borrow();
        serde_json::json!({
            "frames": s.frames,
            "frame_ms": s.frame_ms,
            "zoom": s.now[2],
            "text_zoom": s.text_zoom,
            "cut": s.cut,
            "own_fonts": s.own_fonts,
        })
        .to_string()
    }

    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|p| p.message())
    }
}

/// Place a scene with heights the caller measured, and give the placing back as JSON. The page
/// uses this to check that its own placing and this one are the same rules.
#[wasm_bindgen]
pub fn layout_with_heights(scene_json: &str, heights_json: &str) -> Result<String, JsValue> {
    let scene: Scene = serde_json::from_str(scene_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let heights: Heights = serde_json::from_str(heights_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&scene::layout(&scene, &heights)).map_err(|e| JsValue::from_str(&e.to_string()))
}
