use std::{
	borrow::Cow,
	cell::RefCell,
	cmp::Ordering,
	hash::{DefaultHasher, Hash, Hasher},
	ops::Range,
	rc::Rc,
};

use raylib::{math::Vector2, prelude::RaylibDraw};

use super::{measure_line, Font, DEFAULT_COLOR, SPACING};
use crate::{Color, Details, Layable};

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

				word_wrapping_strategy(&self.text, self.size, &mut wrap_data.lines, det, scale);
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

/// expects lines to be empty already
fn basic_wrapping_strategy(
	text: &str,
	size: i32,
	lines: &mut Vec<Range<usize>>,
	det: Details,
	scale: f32,
) {
	//* caveats:
	// - fix size per character
	// - hardcoded scaling multiple
	// - doesn't adjust to font size

	let chars_per_line = det.aw as f32 / (size as f32 * scale) * 2.0;
	let chars_per_line = chars_per_line.max(1.0) as usize;

	let mut i = 0;
	while i < text.len() {
		let until = i + chars_per_line;
		let until = until.min(text.len());

		let rng = i..until;
		lines.push(rng);
		i = until;
	}
}

use super::word_wrap::word_wrapping_strategy;

/// expects lines to be empty already
///
/// accurately calculates available space for the characters, by incrementally
/// caluclating the text's size with [measure_line] \
/// much slower than basic_wrapping_strategy
fn precise_wrapping_strategy(
	text: &str,
	size: i32,
	lines: &mut Vec<Range<usize>>,
	det: Details,
	scale: f32,
) {
	//* caveats:
	// - computationally expensive on det/scale change

	let real_size = size as f32 * scale;
	let real_size = real_size as i32;

	let mut from = 0;
	let mut to = 0;

	loop {
		if to > text.len() {
			break;
		}

		let test_line = &text[from..to];
		let (width, _) = measure_line(test_line, real_size);

		match width.cmp(&det.aw) {
			Ordering::Greater => {
				match to - from {
					0 => {
						to += 1;
						lines.push(from..to);
						from = to;
						to += 1;
					}
					1 => {
						lines.push(from..to);
						from = to;
						to += 1;
					}
					_ => {
						// we went too far
						lines.push(from..(to - 1));
						from = to - 1;
						to = from + 1;
					}
				}
			}
			Ordering::Equal => {
				lines.push(from..to);
				from = to;
				to += 1;
			}
			Ordering::Less => {
				to += 1;
			}
		}

		if to == text.len() {
			lines.push(from..to)
		}
	}
}
