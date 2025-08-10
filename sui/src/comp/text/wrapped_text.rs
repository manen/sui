use std::{
	borrow::Cow,
	cell::RefCell,
	hash::{DefaultHasher, Hash, Hasher},
	ops::{Deref, DerefMut, Range},
	rc::Rc,
};

use raylib::{math::Vector2, prelude::RaylibDraw};

use super::{measure_line, word_wrap, Font, DEFAULT_COLOR, SPACING};
use crate::{comp::Centered, Color, Details, Layable, LayableExt};

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

	fn recalculate(&mut self, text: &str, size: i32, det: Details, scale: f32) {
		let hash = WrapData::hash(det, scale);
		if self.hash != hash {
			self.force_recalculate(text, size, det, scale);
		}
	}
	fn force_recalculate(&mut self, text: &str, size: i32, det: Details, scale: f32) {
		{
			{
				self.lines.drain(..).for_each(std::mem::drop);

				// we have to get smart to handle static line breaks well
				// we just let the strategy do its thing into the main vec, but we take those elements out
				// and transform them and shit

				// let static_line = text;

				for static_line_rng in word_wrap::text_splitter(text, &[b'\n']) {
					let static_line = &text[static_line_rng.clone()];

					let len_before = self.lines.len();

					word_wrap::word_wrapping_strategy(
						static_line,
						size,
						&mut self.lines,
						det,
						scale,
					);

					for (i, line) in self.lines.iter_mut().enumerate() {
						if i < len_before {
							continue;
						}
						line.start += static_line_rng.start;
						line.end += static_line_rng.start;
					}
				}

				// if for any reason the last line doesn't end on the end of the entire text,
				// add a new line with just the remaining letters
				if let Some(last_rng) = self.lines.iter().cloned().rev().next() {
					if last_rng.end != text.len() {
						self.lines.push(last_rng.end..text.len());
					}
				}
			}

			let (mut width, mut height) = (0, 0);
			for line in self.lines.iter().cloned() {
				let line = &text[line];
				let (line_width, line_height) = measure_line(line, size);
				width = width.min(line_width);
				height += line_height;
			}

			self.width = width;
			self.height = height;

			let hash = WrapData::hash(det, scale);
			self.hash = hash;
		}
	}
}

/// the sibling of [`Text`](crate::comp::Text), with text wrapping enabled, meaning the text
/// will always (at least try to) fit into the space provided. \
///
/// wrapping can be achieved by several strategies, all varying by usecase and performance.
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
		Self::new_explicit(text, size, Font::default(), color)
	}
	pub fn new_explicit<I: Into<Cow<'a, str>>>(
		text: I,
		size: i32,
		font: Font,
		color: Color,
	) -> Self {
		let text = text.into();
		let wrap_data = WrapData::default();
		let wrap_data = Rc::new(RefCell::new(wrap_data));

		Self {
			text,
			size,
			font,
			color,
			wrap_data,
		}
	}

	fn recalculate(&self, det: Details, scale: f32) {
		let mut wrap_data = self.wrap_data.borrow_mut();
		wrap_data.recalculate(&self.text, self.size, det, scale)
	}
}

impl<'a> Layable for WrappedText<'a> {
	fn size(&self) -> (i32, i32) {
		let wrap_data = self.wrap_data.borrow();
		(wrap_data.width, wrap_data.height)
	}

	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.recalculate(det, scale);

		self.font.with_font(|font| {
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
		});
	}
}

#[derive(Debug, Clone)]
pub struct CenteredWrappedText<'a> {
	pub text: Cow<'a, str>,
	pub size: i32,
	font: Font,
	color: Color,

	wrap_data: Rc<
		RefCell<
			// (
			WrapData, // , Vec<Centered<crate::Text<'a>>>)
		>,
	>,
}
impl<'a> CenteredWrappedText<'a> {
	pub fn new<I: Into<Cow<'a, str>>>(text: I, size: i32) -> Self {
		Self::new_colored(text, size, DEFAULT_COLOR)
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
		let wrap_data = Default::default();

		Self {
			text,
			size,
			font,
			color,
			wrap_data,
		}
	}

	fn recalculate(&self, det: Details, scale: f32) {
		let mut wrap_data = self.wrap_data.borrow_mut();
		// let (wrap_data, lines) = wrap_data.deref_mut();

		wrap_data.recalculate(&self.text, self.size, det, scale);

		// let lines = {
		// 	lines.drain(..).for_each(std::mem::drop);

		// 	let lines_ui = wrap_data.lines.iter().cloned().map(|rng| {
		// 		let line = &self.text[rng];
		// 		let line = crate::Text::new(self.text.ra, self.size);

		// 		let line = line.centered();
		// 		line
		// 	});
		// 	lines.extend(lines_ui);
		// };
	}
}
impl<'a> Layable for CenteredWrappedText<'a> {
	fn size(&self) -> (i32, i32) {
		let wrap_data = self.wrap_data.borrow();
		// let wrap_data = &wrap_data.deref().0;
		(wrap_data.width, wrap_data.height)
	}

	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		self.recalculate(det, scale);

		{
			let wrap_data = self.wrap_data.borrow();

			let lines = wrap_data.lines.iter().cloned().map(|rng| &self.text[rng]);
			let lines = lines.map(|line| crate::Text::new(line, self.size).centered());

			let lines = lines.collect::<Vec<_>>();
			let lines = crate::div(lines);

			lines.render(d, det, scale)
		}
	}
}
