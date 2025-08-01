use raylib::prelude::RaylibDraw;

use crate::{
	core::{Event, ImmutableWrap, MouseEvent},
	Details, Layable,
};

use super::Crop;

pub const SCROLLBAR_WIDTH: f32 = 10.0; // it's getting multiplied by scale anyway so we just savin a step
pub const SCROLLBAR_LENGTH: f32 = SCROLLBAR_WIDTH * 4.0;
const SCROLLBAR_BG_COLOR: raylib::color::Color = crate::color(33, 35, 38, 255);
const SCROLLBAR_HANDLE_COLOR: raylib::color::Color = crate::color(106, 113, 122, 255);

const DEBUG: bool = false;
const DEBUG_SCROLLBAR: bool = false;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrollableMode {
	Neither,
	Vertical,
	Horizontal,
	Both,
}
impl ScrollableMode {
	/// (vertical, horizontal)
	fn multipliers(&self) -> (i32, i32) {
		match self {
			ScrollableMode::Neither => (0, 0),
			ScrollableMode::Vertical => (1, 0),
			ScrollableMode::Horizontal => (0, 1),
			ScrollableMode::Both => (1, 1),
		}
	}
	/// (vertical, horizontal)
	fn multipliers_f32(&self) -> (f32, f32) {
		let (x, y) = self.multipliers();
		(x as f32, y as f32)
	}
	/// (vertical, horizontal)
	fn bools(&self) -> (bool, bool) {
		match self {
			ScrollableMode::Neither => (false, false),
			ScrollableMode::Vertical => (true, false),
			ScrollableMode::Horizontal => (false, true),
			ScrollableMode::Both => (true, true),
		}
	}
}

/// ScrollableData stores the data needed for a scrollable to actually scroll
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct ScrollableState {
	// unscaled pixels scrolled in each direction, from 0 to big_width/big_height-small_width/small_height
	pub scroll_x: i32,
	pub scroll_y: i32,

	pub action: ScrollbarAction,
}
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum ScrollbarAction {
	#[default]
	None,
	ScrollingXFrom {
		before: i32,
		drag_start_c: i32,
	}, // the screen x coord the scroll was initiated from
	ScrollingYFrom {
		before: i32,
		drag_start_c: i32,
	}, // the screen y coord the scroll was initiated from
}

#[derive(Clone, Debug)]
pub struct Scrollable<L: Layable> {
	state: ScrollableState,
	mode: ScrollableMode,
	layable: L,
}
impl<L: Layable> Scrollable<L> {
	/// will crop content outside boundaries
	pub fn new(state: ScrollableState, mode: ScrollableMode, layable: L) -> Scrollable<Crop<L>> {
		Scrollable::new_uncropped(state, mode, Crop::new(layable))
	}
	pub fn new_uncropped(state: ScrollableState, mode: ScrollableMode, layable: L) -> Self {
		Self {
			state,
			mode,
			layable,
		}
	}

	fn view(&self, scale: f32) -> View<ImmutableWrap<L>> {
		let (scroll_x, scroll_y) = (self.state.scroll_x, self.state.scroll_y);

		View::new(
			ImmutableWrap::new(&self.layable),
			(scroll_x as f32 * scale) as i32,
			(scroll_y as f32 * scale) as i32,
		)
	}
	fn view_mut(&mut self, scale: f32) -> View<&mut L> {
		// self.clamp(None);
		let (scroll_x, scroll_y) = (self.state.scroll_x, self.state.scroll_y);

		View::new(
			&mut self.layable,
			(scroll_x as f32 * scale) as i32,
			(scroll_y as f32 * scale) as i32,
		)
	}
	/// the det view is rendered with
	fn l_det(&self, det: crate::Details, scale: f32, l_size: Option<(i32, i32)>) -> crate::Details {
		let (l_w, l_h) = l_size.unwrap_or_else(|| self.layable.size());

		let (x_mul, y_mul) = self.mode.bools();

		let x_mul = x_mul && l_w > det.aw;
		let y_mul = y_mul && l_h > det.ah;

		let (x_mul, y_mul) = (if x_mul { 1.0 } else { 0.0 }, if y_mul { 1.0 } else { 0.0 });

		crate::Details {
			x: det.x,
			y: det.y,
			aw: det.aw - (x_mul * SCROLLBAR_WIDTH * scale) as i32,
			ah: det.ah - (y_mul * SCROLLBAR_WIDTH * scale) as i32,
		}
	}

	fn for_each_scrollbar(
		&self,
		l_size: Option<(i32, i32)>,
		view_det: crate::Details,
		scale: f32,
		mut f: impl FnMut((i32, i32, i32, i32), (i32, i32, i32, i32), bool), // last arg is whether the scrollbar's on the bottom
	) {
		let (l_w, l_h) = l_size.unwrap_or_else(|| self.layable.size());
		let (scrollbar_at_side, scrollbar_at_bottom) = self.mode.bools();

		let scrollbar_at_side = scrollbar_at_side && l_h > view_det.ah;
		let scrollbar_at_bottom = scrollbar_at_bottom && l_w > view_det.aw;

		if DEBUG_SCROLLBAR {
			dbg!(scrollbar_at_side, scrollbar_at_bottom);
		}

		if scrollbar_at_side {
			let (scrollbar_base_x, scrollbar_base_y) = (view_det.x + l_w, view_det.y);
			let (scrollbar_w, scrollbar_h) = ((SCROLLBAR_WIDTH * scale) as i32, view_det.ah);

			let (_, scroll_y) = (self.state.scroll_x, self.state.scroll_y);

			let (scrollbar_handle_w, scrollbar_handle_h) = (
				(SCROLLBAR_WIDTH * scale) as i32,
				(SCROLLBAR_LENGTH * scale) as i32,
			);
			let (scrollbar_handle_base_x, scrollbar_handle_base_y) = (
				scrollbar_base_x,
				scrollbar_base_y
					+ (scroll_y as f32 / (l_h - view_det.ah) as f32
						* (view_det.ah as f32 - scrollbar_handle_h as f32)) as i32,
			);
			f(
				(scrollbar_base_x, scrollbar_base_y, scrollbar_w, scrollbar_h),
				(
					scrollbar_handle_base_x,
					scrollbar_handle_base_y,
					scrollbar_handle_w,
					scrollbar_handle_h,
				),
				false,
			);
		}
		if scrollbar_at_bottom {
			let (scrollbar_base_x, scrollbar_base_y) = (view_det.x, view_det.y + l_h);
			let (scrollbar_w, scrollbar_h) = (view_det.aw, (SCROLLBAR_WIDTH * scale) as i32);

			let (scroll_x, _) = (self.state.scroll_x, self.state.scroll_y);

			let (scrollbar_handle_w, scrollbar_handle_h) = (
				// ((l_w as f32 - det.aw as f32) / l_w as f32 * det.aw as f32 * scale) as i32,
				(SCROLLBAR_LENGTH * scale) as i32,
				(SCROLLBAR_WIDTH * scale) as i32,
			);
			let (scrollbar_handle_base_x, scrollbar_handle_base_y) = (
				scrollbar_base_x
					+ (scroll_x as f32 / (l_w - view_det.aw) as f32
						* (view_det.aw as f32 - scrollbar_handle_w as f32)) as i32,
				scrollbar_base_y,
			);
			f(
				(scrollbar_base_x, scrollbar_base_y, scrollbar_w, scrollbar_h),
				(
					scrollbar_handle_base_x,
					scrollbar_handle_base_y,
					scrollbar_handle_w,
					scrollbar_handle_h,
				),
				true,
			);
		}
	}
	fn clamp(&mut self, det: Details, l_size: Option<(i32, i32)>) {
		let (vert, horiz) = self.mode.bools();
		let x_off = if horiz { det.aw } else { 0 };
		let y_off = if vert { det.ah } else { 0 };

		let (l_w, l_h) = l_size.unwrap_or_else(|| self.layable.size());

		self.state.scroll_x = self.state.scroll_x.min(l_w - x_off).max(0);
		self.state.scroll_y = self.state.scroll_y.min(l_h - y_off).max(0);
	}
}
impl<L: Layable> Layable for Scrollable<L> {
	/// returns self.layable.size(), because this scrollable is likely in a FixedSize, so it doesn't really matter
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		let (l_w, l_h) = self.layable.size();

		let view = self.view(scale);
		let view_det = self.l_det(det, scale, Some((l_w, l_h)));
		view.render(d, view_det, scale);

		self.for_each_scrollbar(
			Some((l_w, l_h)),
			view_det,
			scale,
			|(scrollbar_base_x, scrollbar_base_y, scrollbar_w, scrollbar_h),
			 (handle_base_x, handle_base_y, handle_w, handle_h),
			 _| {
				if DEBUG_SCROLLBAR {
					dbg!(scrollbar_base_x, scrollbar_base_y, scrollbar_w, scrollbar_h);
				}

				d.draw_rectangle(
					scrollbar_base_x,
					scrollbar_base_y,
					scrollbar_w,
					scrollbar_h,
					SCROLLBAR_BG_COLOR,
				);
				d.draw_rectangle(
					handle_base_x,
					handle_base_y,
					handle_w,
					handle_h,
					SCROLLBAR_HANDLE_COLOR,
				);
			},
		);

		if DEBUG {
			d.draw_text(
				&format!("lsize: {:?}\ndet: {det:?}\nvdet: {view_det:?}", (l_w, l_h)),
				det.x,
				det.y,
				12,
				crate::Color::WHEAT,
			);
		}
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
		let (mul_x, mul_y) = self.mode.multipliers_f32();
		let (l_w, l_h) = self.layable.size();

		let view_det = self.l_det(det, scale, Some((l_w, l_h)));

		let events = events.collect::<Vec<_>>();
		// different events do different things:
		// - pressing once starts the action
		// - MouseHeld updates self.scroll_x or self.scroll_y
		// - release stops the action
		for event in events.iter().copied() {
			match event {
				Event::MouseEvent(MouseEvent::Scroll { amount, .. }) => {
					if self.mode == ScrollableMode::Horizontal {
						self.state.scroll_x -= (amount * 10.0) as i32;
					} else {
						self.state.scroll_y -= (amount * 10.0) as i32;
					}
					self.clamp(det, None);
				}
				Event::MouseEvent(MouseEvent::MouseClick {
					x: mouse_x,
					y: mouse_y,
				}) => {
					let mut state = self.state;
					self.for_each_scrollbar(
						Some((l_w, l_h)),
						view_det,
						scale,
						|_, handle, bottom| {
							let handle_det = crate::Details {
								x: handle.0,
								y: handle.1,
								aw: handle.2,
								ah: handle.3,
							};

							if handle_det.is_inside(mouse_x, mouse_y) {
								state.action = if bottom {
									ScrollbarAction::ScrollingXFrom {
										before: state.scroll_x,
										drag_start_c: mouse_x,
									}
								} else {
									ScrollbarAction::ScrollingYFrom {
										before: state.scroll_y,
										drag_start_c: mouse_y,
									}
								}
							}
						},
					);
					self.state = state;
				}
				Event::MouseEvent(MouseEvent::MouseHeld {
					x: mouse_x,
					y: mouse_y,
				}) => {
					match self.state.action {
						ScrollbarAction::ScrollingXFrom {
							before,
							drag_start_c,
						} => {
							let og = before
								+ ((mouse_x - drag_start_c) as f32
									/ (view_det.aw as f32 - SCROLLBAR_LENGTH * scale)
									* (l_w - view_det.aw) as f32) as i32;

							// i know this is a little unreadable :(
							// basically all this math is just the inverse of how we calculate where the scrollbar handle should be
							// same for ScrollingFromY

							self.state.scroll_x = og
								.min(l_w - det.aw + (SCROLLBAR_WIDTH * scale * mul_x) as i32)
								.max(0);
						}
						ScrollbarAction::ScrollingYFrom {
							before,
							drag_start_c,
						} => {
							let og = before
								+ ((mouse_y - drag_start_c) as f32
									/ (view_det.ah as f32 - SCROLLBAR_LENGTH * scale)
									* (l_h - view_det.ah) as f32) as i32;

							self.state.scroll_y = og
								.min(l_h - det.ah + (SCROLLBAR_WIDTH * scale * mul_y) as i32)
								.max(0);
						}
						_ => (), // no action has been started
					};
				}
				Event::MouseEvent(MouseEvent::MouseRelease { .. }) => {
					// expects everything to be handled in Event::MouseHeld
					self.state.action = ScrollbarAction::None;
				}
				_ => (),
			}
		}

		let mut view = self.view_mut(scale);
		view.pass_events(events.into_iter(), view_det, scale)
			.collect::<Vec<_>>()
			.into_iter()
	}
}

#[derive(Clone, Debug)]
/// Renders `self.layable`, with an offset that it will appear as though `self.layable` is rendering from `(self.base_x, self.base_y)`
///
/// does not currently crop the content. this whole struct is basically just to make making scrollables crop their content easier
pub struct View<L: Layable> {
	layable: L,
	base_x: i32,
	base_y: i32,
}
impl<L: Layable> View<L> {
	pub fn new(layable: L, x: i32, y: i32) -> Self {
		Self {
			layable,
			base_x: x,
			base_y: y,
		}
	}
	pub fn take(self) -> L {
		self.layable
	}

	pub fn l_det(&self, det: crate::Details, _scale: f32) -> crate::Details {
		crate::Details {
			x: det.x - self.base_x,
			y: det.y - self.base_y,
			aw: det.aw + self.base_x,
			ah: det.ah + self.base_y,
		}
	}
}
impl<L: Layable> Layable for View<L> {
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
	) -> impl Iterator<Item = crate::core::ReturnEvent> {
		let event_f = |event| match event {
			Event::KeyboardEvent(_, _) => event,
			Event::MouseEvent(a) => Event::MouseEvent(
				a.with_cursor_pos_transform(|(x, y)| (x + self.base_x, y + self.base_y)),
			),
		};

		let events = events.map(event_f);
		self.layable
			.pass_events(events, self.l_det(det, scale), scale)
	}
}
