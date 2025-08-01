pub use raylib;

pub mod core;
pub use core::{Details, DynamicLayable, Handle, Layable};

pub mod comp;
pub use comp::{Comp, Compatible, Div, Text};

pub mod dialog;

pub mod tex;

pub mod form;

pub(crate) mod root_ctx;
pub mod ui;
pub use ui::*;

pub type Color = raylib::color::Color;
pub const fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
	raylib::color::Color { r, g, b, a }
}
