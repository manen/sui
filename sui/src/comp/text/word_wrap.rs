use std::{cmp::Ordering, ops::Range};

use crate::{comp::text::measure_line, Details};

pub fn word_is_line_startegy(
	text: &str,
	_size: i32,
	lines: &mut Vec<Range<usize>>,
	_det: Details,
	_scale: f32,
) {
	let triggers = [b' '];
	let split = text_splitter(text, &triggers);

	for word in split {
		lines.push(word);
	}
}

pub fn word_wrapping_strategy(
	text: &str,
	size: i32,
	lines: &mut Vec<Range<usize>>,
	det: Details,
	scale: f32,
) {
	let real_size = size as f32 * scale;
	let real_size = real_size as i32;

	let triggers = [b' '];
	let words = text_splitter(text, &triggers);

	let mut line_from = 0;
	let mut last_was_less = false;
	for rng in words.iter() {
		last_was_less = false;

		let line = &text[line_from..rng.end];
		let (width, _) = measure_line(line, real_size);

		// if line.ends_with('\n') {
		// 	lines.push(line_from..rng.end - 1);
		// 	line_from = rng.end;
		// 	continue;
		// }

		match width.cmp(&det.aw) {
			Ordering::Greater => {
				// we went too far
				lines.push(line_from..rng.start);
				line_from = rng.start;
			}
			Ordering::Equal => {
				lines.push(line_from..rng.end);
				line_from = rng.end;
			}
			Ordering::Less => {
				// ok
				last_was_less = true;
			}
		}
	}
	if last_was_less {
		lines.push(line_from..text.len())
	}
}

/// splits text into ranges, separated by any of the triggers
pub(crate) fn text_splitter<'a>(text: &'a str, triggers: &'a [u8]) -> Vec<Range<usize>> {
	let size = text.bytes().filter(|c| triggers.contains(c)).count() as usize + 1;
	let mut buf = Vec::with_capacity(size);

	let mut from = 0;
	for (i, c) in text.bytes().enumerate() {
		if triggers.contains(&c) {
			buf.push(from..i);
			from = i + 1;
		}
	}

	if buf.len() < size {
		buf.push(from..text.len())
	}
	buf
}

/// expects lines to be empty already
pub fn basic_wrapping_strategy(
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

/// expects lines to be empty already
///
/// accurately calculates available space for the characters, by incrementally
/// caluclating the text's size with [measure_line] \
/// much slower than basic_wrapping_strategy
pub fn precise_wrapping_strategy(
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
