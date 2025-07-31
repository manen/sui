use crate::{core::Event, Layable};

#[derive(Clone, Debug)]
pub struct Scale<L: Layable> {
	layable: L,
	scale: f32,
}
impl<L: Layable> Scale<L> {
	pub fn new(layable: L, scale: f32) -> Self {
		Self { layable, scale }
	}
}
impl<L: Layable> Layable for Scale<L> {
	fn size(&self) -> (i32, i32) {
		let (lw, lh) = self.layable.size();
		let (w, h) = (lw as f32 * self.scale, lh as f32 * self.scale);
		(w as i32, h as i32)
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, det, scale * self.scale);
	}

	fn tick(&mut self) {
		self.layable.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: crate::Details,
		scale: f32,
	) -> impl Iterator<Item = crate::core::ReturnEvent> {
		let map_event = |event| match event {
			Event::MouseEvent(m) => Event::MouseEvent(m.with_cursor_pos_transform(|(x, y)| {
				(
					(x as f32 / self.scale) as i32,
					(y as f32 / self.scale) as i32,
				)
			})),
			_ => event,
		};
		self.layable.pass_events(events.map(map_event), det, scale)
	}
}
