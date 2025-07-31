use crate::{core::Event, Layable};

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
	) -> impl Iterator<Item = crate::core::ReturnEvent> {
		let mut ret = Vec::with_capacity(events.size_hint().0);

		for event in events {
			if let Some(ret_event) = self
				.foreground
				.pass_events(std::iter::once(event), det, scale)
				.next()
			{
				ret.push(ret_event);
			} else {
				if let Some(ret_event) = self
					.background
					.pass_events(std::iter::once(event), det, scale)
					.next()
				{
					ret.push(ret_event);
				}
			}
		}
		ret.into_iter()
	}
}
