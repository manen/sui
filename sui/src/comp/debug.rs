use raylib::prelude::RaylibDraw;

use crate::{core::Event, Layable};

#[derive(Clone, Debug)]
/// `Debug` renders some useful ui debug info: \
///
/// - layable.size() in red
/// - self.render det in blue
pub struct Debug<L: Layable> {
	layable: L,
}
impl<L: Layable> Debug<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}
}
impl<L: Layable> Layable for Debug<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		use raylib::color::Color;

		let size = self.layable.size();
		d.draw_rectangle_lines(
			det.x,
			det.y,
			(size.0 as f32 * scale) as _,
			(size.1 as f32 * scale) as _,
			Color::RED,
		);

		d.draw_rectangle_lines(
			det.x,
			det.y,
			(det.aw as f32 * scale) as _,
			(det.ah as f32 * scale) as _,
			Color::BLUE,
		);

		self.layable.render(d, det, scale)
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
		self.layable.pass_events(events, det, scale)
	}
}
