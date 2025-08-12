use crate::{
	core::{Event, MouseEvent, ReturnEvent},
	Layable,
};
use std::fmt::Debug;

#[derive(Clone)]
/// while this technically does work with any Layable, to implement Compatible C needs to be Comp
pub struct Clickable<C, F, T>
where
	T: 'static,
	F: FnMut((i32, i32)) -> T,
	C: Layable,
{
	comp: C,
	gen_ret: F,
	/// if true, it will check if self.comp bubbles anything back and only respond if it doesn't
	fallback: bool,
}
impl<C: Layable + std::fmt::Debug, T, F: FnMut((i32, i32)) -> T> std::fmt::Debug
	for Clickable<C, F, T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Clickable")
			.field("comp", &self.comp)
			.field("fallback", &self.fallback)
			.finish()
	}
}
impl<C: Layable, T, F: FnMut((i32, i32)) -> T> Clickable<C, F, T> {
	pub fn new(gen_ret: F, comp: C) -> Self {
		Clickable {
			comp,
			gen_ret,
			fallback: false,
		}
	}
	pub fn new_fallback(gen_ret: F, comp: C) -> Self {
		Clickable {
			comp,
			gen_ret,
			fallback: true,
		}
	}

	pub fn take(self) -> C {
		self.comp
	}
}
impl<T, C: Layable, F: FnMut((i32, i32)) -> T> Layable for Clickable<C, F, T> {
	fn size(&self) -> (i32, i32) {
		self.comp.size()
	}

	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.comp.render(d, det, scale)
	}

	fn tick(&mut self) {
		self.comp.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: crate::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let mut f = move |event| {
			let mut respond = || match event {
				Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
					if det.is_inside(x, y) {
						Some(Event::ret((self.gen_ret)((x, y))))
					} else {
						None
					}
				}
				_ => None,
			};

			let mut testing = Vec::new();
			if !self.fallback {
				match respond() {
					Some(a) => ret_events.push(a),
					None => {
						self.comp
							.pass_events(std::iter::once(event), det, scale, &mut testing);
						if let Some(a) = testing.drain(..).next() {
							ret_events.push(a);
						}
					}
				}
			} else {
				if let Some(comp_resp) = {
					self.comp
						.pass_events(std::iter::once(event), det, scale, &mut testing);
					testing.drain(..).next()
				} {
					ret_events.push(comp_resp)
				} else {
					if let Some(a) = respond() {
						ret_events.push(a)
					}
				}
			}
		};

		for event in events {
			f(event)
		}
	}
}

// ---

#[derive(Clone)]
/// an alternate Clickable variant that:
///
/// 1. allows you to use a closure that has a static return type
/// 2. allows optional return event generation
pub struct OptionalClickable<L: Layable, F: FnMut() -> Option<ReturnEvent>> {
	layable: L,
	gen_f: F,
}
impl<L: Layable, F: FnMut() -> Option<ReturnEvent>> OptionalClickable<L, F> {
	pub fn new(layable: L, gen_f: F) -> Self {
		Self { layable, gen_f }
	}
}
impl<L: Layable + Debug, F: FnMut() -> Option<ReturnEvent>> Debug for OptionalClickable<L, F> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("NeoClickable")
			.field("layable", &self.layable)
			.finish()
	}
}
impl<L: Layable, F: FnMut() -> Option<ReturnEvent>> Layable for OptionalClickable<L, F> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, det, scale);
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
		for event in events {
			match event {
				Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
					if det.is_inside(x, y) {
						let generated = (self.gen_f)();
						if let Some(generated) = generated {
							ret_events.push(generated)
						}
					}
				}
				_ => {}
			}
		}
	}
}
