mod dyn_layable;
pub use dyn_layable::DynamicLayable;
use raylib::RaylibHandle;
use std::fmt::Debug;

mod store;
pub use store::{Cached, Store};

mod event;
pub use event::*;

mod handle;
pub use handle::*;

mod immutable_wrap;
pub use immutable_wrap::ImmutableWrap;

pub trait Layable {
	fn size(&self) -> (i32, i32);
	fn render(&self, d: &mut Handle, det: Details, scale: f32);

	fn tick(&mut self) {}

	/// this is what other layables should implement
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let _ = (events, det, scale, ret_events);
	}
}
impl<L: Layable> Layable for &mut L {
	fn size(&self) -> (i32, i32) {
		L::size(self)
	}
	fn render(&self, d: &mut Handle, det: Details, scale: f32) {
		L::render(self, d, det, scale)
	}
	fn tick(&mut self) {
		L::tick(self)
	}
	fn pass_events(
		&mut self,
		event: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		L::pass_events(self, event, det, scale, ret_events)
	}
}

#[derive(Copy, Clone, Debug, Hash, Default)]
pub struct Details {
	pub x: i32,
	pub y: i32,
	/// available width
	pub aw: i32,
	/// available height
	pub ah: i32,
}
impl Details {
	pub fn new(x: i32, y: i32, aw: i32, ah: i32) -> Self {
		Self { x, y, aw, ah }
	}
	pub fn window(w: i32, h: i32) -> Self {
		Self::new(0, 0, w, h)
	}
	pub fn rl_window(rl: &RaylibHandle) -> Self {
		Self::window(rl.get_render_width(), unsafe {
			raylib::ffi::GetRenderHeight()
		})
	}

	pub fn from_top(&self, h: i32) -> Self {
		Self {
			x: self.x,
			y: self.y,
			aw: self.aw,
			ah: h,
		}
	}
	pub fn from_bottom(&self, h: i32) -> Self {
		Self {
			x: self.x,
			y: self.y + self.ah - h,
			aw: self.aw,
			ah: h,
		}
	}
	pub fn from_left(&self, w: i32) -> Self {
		Self {
			x: self.x,
			y: self.y,
			aw: w,
			ah: self.ah,
		}
	}
	pub fn from_right(&self, w: i32) -> Self {
		Self {
			x: self.x + self.aw - w,
			y: self.y,
			aw: w,
			ah: self.ah,
		}
	}

	pub fn split_v(&self, pieces: i32) -> impl Iterator<Item = Self> {
		let one_w = self.aw / pieces;
		let base_x = self.x;
		let y = self.y;
		let ah = self.ah;

		(0..pieces).map(move |i| one_w * i).map(move |x| Self {
			x: base_x + x,
			y,
			aw: one_w,
			ah,
		})
	}
	pub fn split_h(&self, pieces: i32) -> impl Iterator<Item = Self> {
		let one_h = self.ah / pieces;
		let base_y = self.y;
		let x = self.x;
		let aw = self.aw;

		(0..pieces).map(move |i| one_h * i).map(move |y| Self {
			x,
			y: base_y + y,
			aw,
			ah: one_h,
		})
	}

	pub fn mul_size(self, scale: f32) -> Self {
		Self {
			aw: (self.aw as f32 * scale) as _,
			ah: (self.ah as f32 * scale) as _,
			..self
		}
	}

	pub fn is_inside(&self, x: i32, y: i32) -> bool {
		x >= self.x && x <= self.x + self.aw // x
			&& y >= self.y && y <= self.y + self.ah
	}
	pub fn is_inside_tuple(&self, (x, y): (i32, i32)) -> bool {
		x >= self.x && x <= self.x + self.aw // x
			&& y >= self.y && y <= self.y + self.ah
	}

	/// returns true if the two details share any area
	pub fn intersects(&self, rhs: &Self) -> bool {
		let (a, b) = (self, rhs);

		let r1_left = a.x;
		let r1_right = a.x + a.aw;
		let r1_bottom = a.y;
		let r1_top = a.y + a.ah;

		let r2_left = b.x;
		let r2_right = b.x + b.aw;
		let r2_bottom = b.y;
		let r2_top = b.y + b.ah;

		let no_overlap = r1_right <= r2_left
			|| r2_right <= r1_left
			|| r1_top <= r2_bottom
			|| r2_top <= r1_bottom;

		!no_overlap
	}
}
