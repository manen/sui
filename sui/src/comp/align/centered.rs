use crate::{
	core::{Event, ReturnEvent},
	Details, Layable,
};

#[derive(Clone, Debug)]
/// self.size() is self.layable.size(), the centering only happens on self.render()
pub struct Centered<L: Layable> {
	layable: L,
}
impl<L: Layable> Centered<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}

	fn l_det(&self, det: Details, scale: f32) -> Details {
		let (l_w, l_h) = self.layable.size();
		let (l_w, l_h) = ((l_w as f32 * scale), (l_h as f32 * scale));

		let (x_offset, y_offset) = (
			(det.aw as f32 / 2.0 - l_w / 2.0) as i32,
			(det.ah as f32 / 2.0 - l_h / 2.0) as i32,
		);
		let l_det = crate::Details {
			x: det.x + x_offset,
			y: det.y + y_offset,
			aw: det.aw - x_offset,
			ah: det.ah - y_offset,
			// aw: l_w as i32,
			// ah: l_h as i32,
		};

		l_det
	}
}
impl<L: Layable> Layable for Centered<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale);
	}

	fn tick(&mut self) {
		self.layable.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: crate::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		self.layable
			.pass_events(events, self.l_det(det, scale), scale, ret_events)
	}
}
