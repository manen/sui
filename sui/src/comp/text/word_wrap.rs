use std::{cmp::Ordering, ops::Range};

use crate::{comp::text::measure_line, Details};

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
		let line = &text[line_from..rng.end];
		let (width, _) = measure_line(line, real_size);
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

fn text_splitter<'a>(text: &'a str, triggers: &'a [u8]) -> Vec<Range<usize>> {
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
