use std::{
	borrow::Cow,
	cell::RefCell,
	hash::{DefaultHasher, Hash, Hasher},
	ops::Range,
	rc::Rc,
};

use raylib::{math::Vector2, prelude::RaylibDraw};

use crate::{Color, Details, Layable};

use super::text::{measure_line, Font, DEFAULT_COLOR, SPACING};

#[derive(Debug, Default)]
pub struct WrapData {
	hash: u64,

	width: i32,
	height: i32,

	lines: Vec<Range<usize>>,
}
impl WrapData {
	fn hash(det: Details, scale: f32) -> u64 {
		let mut hasher = DefaultHasher::new();
		det.hash(&mut hasher);
		((scale * 40.0) as i32).hash(&mut hasher);
		hasher.finish()
	}
}

#[derive(Debug, Clone)]
pub struct WrappedText<'a> {
	pub text: Cow<'a, str>,
	pub size: i32,
	font: Font,
	color: Color,

	wrap_data: Rc<RefCell<WrapData>>,
}

impl<'a> WrappedText<'a> {
	pub fn new<I: Into<Cow<'a, str>>>(text: I, size: i32) -> Self {
		Self::new_colored(text, size, DEFAULT_COLOR)
	}

	pub fn new_colored<I: Into<Cow<'a, str>>>(text: I, size: i32, color: Color) -> Self {
		let text = text.into();
		let wrap_data = WrapData::default();
		let wrap_data = Rc::new(RefCell::new(wrap_data));

		Self {
			text,
			size,
			font: Font,
			color,
			wrap_data,
		}
	}

	fn recalculate(&self, det: Details, scale: f32) {
		let hash = WrapData::hash(det, scale);
		if self.wrap_data.borrow().hash != hash {
			self.force_recalculate(det, scale);
		}
	}
	fn force_recalculate(&self, det: Details, scale: f32) {
		{
			let mut wrap_data = self.wrap_data.borrow_mut();

			{
				wrap_data.lines.drain(..).for_each(std::mem::drop);

				let half = self.text.len() / 2;

				wrap_data.lines.push(0..half);
				wrap_data.lines.push(half..self.text.len());
			}

			let (mut width, mut height) = (0, 0);
			for line in wrap_data.lines.iter().cloned() {
				let line = &self.text[line];
				let (line_width, line_height) = measure_line(line, self.size);
				width = width.min(line_width);
				height += line_height;
			}

			wrap_data.width = width;
			wrap_data.height = height;

			let hash = WrapData::hash(det, scale);
			wrap_data.hash = hash;
		}
	}
}

impl<'a> Layable for WrappedText<'a> {
	fn size(&self) -> (i32, i32) {
		let wrap_data = self.wrap_data.borrow();
		(wrap_data.width, wrap_data.height)
	}

	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.recalculate(det, scale);

		let font = self.font.get_font_d(d);

		let mut y = det.y;
		for line in self.wrap_data.borrow().lines.iter().cloned() {
			let text = &self.text[line];
			d.draw_text_ex(
				&font,
				text,
				Vector2::new(det.x as f32, y as f32),
				self.size as f32 * scale,
				SPACING,
				self.color,
			);
			y += self.size;
		}
	}
}
