#![allow(clippy::style, clippy::type_complexity)]
pub mod app;
pub mod assets;
pub mod core;
pub mod input;
pub mod renderer;
pub mod stage;
pub mod util;

pub use app::Application;

#[cfg(target_arch = "wasm32")]
pub mod reexport {
    pub use wasm_bindgen;
}
