use crate::core::{Event, Layable, ReturnEvent};
use crate::{
	comp::{Comp, Compatible},
	Details,
};

pub mod space_between;
pub use space_between::SpaceBetween;

pub trait DivComponents: Sized {
	type L: Layable;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L>;
	fn iter_components_mut(&mut self) -> Option<impl Iterator<Item = &mut Self::L>>;
}
impl<const N: usize, L: Layable> DivComponents for [L; N] {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
	fn iter_components_mut(&mut self) -> Option<impl Iterator<Item = &mut Self::L>> {
		Some(self.iter_mut())
	}
}
impl<L: Layable> DivComponents for &[L] {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
	fn iter_components_mut(&mut self) -> Option<impl Iterator<Item = &mut Self::L>> {
		Option::<std::iter::Empty<&mut Self::L>>::None
	}
}
impl<L: Layable> DivComponents for Vec<L> {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
	fn iter_components_mut(&mut self) -> Option<impl Iterator<Item = &mut Self::L>> {
		Some(self.iter_mut())
	}
}
impl<L: Layable> DivComponents for L {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		std::iter::once(self)
	}
	fn iter_components_mut(&mut self) -> Option<impl Iterator<Item = &mut Self::L>> {
		Some(std::iter::once(self))
	}
}

/// simple page layout, one element after another \
/// just imagine an html div
#[derive(Clone, Debug, Default)]
pub struct Div<D: DivComponents> {
	components: D,
	horizontal: bool,
}
impl<D: DivComponents + Default> Div<D> {
	pub fn empty() -> Self {
		Self::default()
	}
	pub fn empty_horizontal() -> Self {
		Self {
			horizontal: true,
			..Default::default()
		}
	}
}
impl<D: DivComponents> Div<D> {
	pub fn new(horizontal: bool, components: D) -> Self {
		Self {
			components: components,
			horizontal,
		}
	}
	pub fn vertical(components: D) -> Self {
		Self::new(false, components)
	}
	pub fn horizontal(components: D) -> Self {
		Self::new(true, components)
	}

	pub fn as_horizontal(self) -> Self {
		Self {
			horizontal: true,
			..self
		}
	}
}
impl<'a, L: Layable> Div<Vec<L>> {
	pub fn empty_with_capacity(capacity: usize) -> Self {
		Self {
			components: Vec::with_capacity(capacity),
			horizontal: false,
		}
	}
	pub fn empty_horizontal_with_capacity(capacity: usize) -> Self {
		Self {
			components: Vec::with_capacity(capacity),
			horizontal: true,
		}
	}

	pub fn push(&mut self, next_layable: L) {
		self.components.push(next_layable)
	}
}
impl<D: DivComponents> Layable for Div<D> {
	fn size(&self) -> (i32, i32) {
		let (mut w, mut h) = (0, 0);

		for comp in self.components.iter_components() {
			let (comp_w, comp_h) = comp.size();

			if !self.horizontal {
				(w, h) = (w.max(comp_w), h + comp_h)
			} else {
				(w, h) = (w + comp_w, h.max(comp_h))
			}
		}

		(w, h)
	}

	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		let (mut x, mut y) = (det.x, det.y);
		for comp in self.components.iter_components() {
			let (comp_w, comp_h) = comp.size();
			let comp_det = Details {
				x,
				y,
				aw: if !self.horizontal {
					(det.aw as f32 * scale) as i32
				} else {
					comp_w
				},
				ah: if self.horizontal {
					(det.ah as f32 * scale) as i32
				} else {
					comp_h
				},
			};

			comp.render(d, comp_det, scale);

			if !self.horizontal {
				y += (comp_h as f32 * scale) as i32;
			} else {
				x += (comp_w as f32 * scale) as i32;
			}
		}
	}

	fn tick(&mut self) {
		if let Some(iter) = self.components.iter_components_mut() {
			for comp in iter {
				comp.tick();
			}
		}
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let (self_w, self_h) = self.size();

		let mut event_f = move |event| match self.components.iter_components_mut() {
			Some(components) => match event {
				Event::MouseEvent(m_event) => {
					let (mouse_x, mouse_y) = m_event.at();

					let (mut x, mut y) = (det.x, det.y);
					for comp in components {
						let (comp_w, comp_h) = comp.size();
						let comp_det = Details {
							x,
							y,
							aw: if !self.horizontal {
								(self_w as f32 * scale) as i32
							} else {
								comp_w
							},
							ah: if self.horizontal {
								(self_h as f32 * scale) as i32
							} else {
								comp_h
							},
						};

						if comp_det.is_inside(mouse_x, mouse_y) {
							comp.pass_events(std::iter::once(event), comp_det, scale, ret_events);
							// TODO mouse coords aren't translated
						}

						if !self.horizontal {
							y += (comp_h as f32 * scale) as i32;
						} else {
							x += (comp_w as f32 * scale) as i32;
						}
					}
				}
				Event::KeyboardEvent(_, _) => {
					for c in components {
						let len_before = ret_events.len();
						c.pass_events(std::iter::once(event), det, scale, ret_events);
						if len_before != ret_events.len() {
							return;
						}
					}
				}
			},
			_ => (),
		};

		for event in events {
			event_f(event)
		}
	}
}

impl<L: Layable> FromIterator<L> for Div<Vec<L>> {
	fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
		let iter = iter.into_iter();
		Div::new(false, iter.collect::<Vec<_>>())
	}
}
