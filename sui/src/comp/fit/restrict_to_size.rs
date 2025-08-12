use crate::{Details, Layable};

#[derive(Clone, Debug)]
/// restricts the rendering of the child layable to the size it requests
pub struct RestrictToSize<L: Layable> {
	layable: L,
}
impl<L: Layable> RestrictToSize<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}

	fn l_det(&self, det: Details, scale: f32) -> Details {
		let (l_w, l_h) = self.size();
		let det = Details {
			aw: (l_w as f32 * scale) as _,
			ah: (l_h as f32 * scale) as _,
			..det
		};

		det
	}
}
impl<L: Layable> Layable for RestrictToSize<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale);
	}

	fn tick(&mut self) {
		self.layable.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = crate::core::Event>,
		det: Details,
		scale: f32,
		ret_events: &mut Vec<crate::core::ReturnEvent>,
	) {
		self.layable
			.pass_events(events, self.l_det(det, scale), scale, ret_events)
	}
}
