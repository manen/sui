use crate::Layable;
use raylib::{color::Color, math::Vector2, prelude::RaylibDraw, text::WeakFont};
use std::{
	borrow::Cow,
	sync::{Arc, Mutex},
};

pub mod wrapped_text;
pub use wrapped_text::{CenteredWrappedText, WrappedText};

pub mod font;
pub(self) mod word_wrap;
pub use font::Font;

// --

pub const BOUNDS_DEBUG: bool = false;
pub const SPACING: f32 = 1.0; // idk idc
pub const DEFAULT_COLOR: Color = Color::WHITE;
// pub const DEFAULT_COLOR: Color = crate::color(195, 36, 209, 255);

#[derive(Debug, Clone)]
pub struct Text<'a> {
	pub text: Cow<'a, str>,
	pub size: i32,
	font: Font,
	color: Color,
}

impl<'a> Text<'a> {
	pub fn new<I: Into<Cow<'a, str>>>(text: I, size: i32) -> Self {
		Self::new_colored(text, size, Color::WHITE)
	}
	pub fn new_colored<I: Into<Cow<'a, str>>>(text: I, size: i32, color: Color) -> Self {
		Self::new_explicit(text, size, Font::default(), color)
	}
	pub fn new_explicit<I: Into<Cow<'a, str>>>(
		text: I,
		size: i32,
		font: Font,
		color: Color,
	) -> Self {
		let text = text.into();
		Self {
			text,
			size,
			font,
			color,
		}
	}
}
impl<'a, I: Into<Cow<'a, str>>> Into<Text<'a>> for (I, i32) {
	fn into(self) -> Text<'a> {
		Text::new(self.0, self.1)
	}
}

impl<'a> Layable for Text<'a> {
	fn size(&self) -> (i32, i32) {
		self.text
			.split('\n')
			.map(|line| measure_line(line, self.size))
			.fold((0, 0), |acc, (x, y)| (acc.0.max(x), acc.1 + y - 1)) // i don't know why we need to remove 1 pixel from height per line
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		if BOUNDS_DEBUG {
			let s = self.size();
			d.draw_rectangle_lines(det.x, det.y, s.0, s.1, Color::WHITE);
		}

		self.font.with_font(|font| {
			d.draw_text_ex(
				font,
				&self.text,
				Vector2::new(det.x as f32, det.y as f32),
				self.size as f32 * scale,
				SPACING,
				self.color,
			);
		})
	}
}

pub fn measure_line(text: &str, size: i32) -> (i32, i32) {
	let font = Font::default();
	measure_line_font(text, size, &font)
}
pub fn measure_line_font(text: &str, size: i32, font: &Font) -> (i32, i32) {
	font.with_font(|font| {
		let cstring =
			std::ffi::CString::new(text).expect("CString::new failed while measuring text size:(");

		let dimensions = unsafe {
			raylib::ffi::MeasureTextEx(
				raylib::ffi::Font::clone(font),
				cstring.as_ptr(),
				size as f32,
				SPACING,
			)
		};

		(dimensions.x.ceil() as i32, dimensions.y.ceil() as i32)
	})
}
