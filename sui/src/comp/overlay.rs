use crate::{
	core::{Event, ReturnEvent},
	Layable,
};

#[derive(Clone, Debug)]
/// renders the two components in the same place, overlapping each other
pub struct Overlay<A: Layable, B: Layable> {
	foreground: A,
	background: B,
}
impl<A: Layable, B: Layable> Overlay<A, B> {
	pub fn new(background: B, foreground: A) -> Self {
		Self {
			foreground,
			background,
		}
	}
}
impl<A: Layable, B: Layable> Layable for Overlay<A, B> {
	fn size(&self) -> (i32, i32) {
		let (a_w, a_h) = self.foreground.size();
		let (b_w, b_h) = self.background.size();

		(a_w.max(b_w), a_h.max(b_h))
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.background.render(d, det, scale);
		self.foreground.render(d, det, scale);
	}

	fn tick(&mut self) {
		self.foreground.tick();
		self.background.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: crate::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let mut testing = Vec::new();
		for event in events {
			self.foreground
				.pass_events(std::iter::once(event), det, scale, &mut testing);

			let testing_first = testing.drain(..).nth(0);
			if let Some(ret_event) = testing_first {
				ret_events.push(ret_event);
			} else {
				self.background
					.pass_events(std::iter::once(event), det, scale, &mut testing);
				if let Some(ret_event) = testing.drain(..).nth(0) {
					ret_events.push(ret_event);
				}
			}
		}
	}
}
