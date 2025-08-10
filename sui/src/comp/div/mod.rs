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
impl<D: DivComponents> Div<D> {
	pub fn for_each<F: FnMut(&D::L, Details)>(&self, det: Details, scale: f32, mut f: F) {
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

			f(comp, comp_det);

			if !self.horizontal {
				y += (comp_h as f32 * scale) as i32;
			} else {
				x += (comp_w as f32 * scale) as i32;
			}
		}
	}
	pub fn for_each_mut<F: FnMut(&mut D::L, Details)>(
		&mut self,
		det: Details,
		scale: f32,
		mut f: F,
	) {
		let (mut x, mut y) = (det.x, det.y);
		for comp in self.components.iter_components_mut().into_iter().flatten() {
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

			f(comp, comp_det);

			if !self.horizontal {
				y += (comp_h as f32 * scale) as i32;
			} else {
				x += (comp_w as f32 * scale) as i32;
			}
		}
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
		self.for_each(det, scale, |comp, l_det| {
			comp.render(d, l_det, scale);
		});
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
		let events = events.collect::<Vec<_>>(); // TODO: make events clone

		self.for_each_mut(det, scale, |comp, l_det| {
			let l_events = events.iter().cloned().filter_map(|event| match event {
				Event::KeyboardEvent(..) => Some(event),
				Event::MouseEvent(m_event) => {
					if l_det.is_inside_tuple(m_event.at()) {
						Some(event)
					} else {
						None
					}
				}
			});
			let l_events = l_events.collect::<Vec<_>>().into_iter(); // TODO: allocations hurt my soul for some reason
			comp.pass_events(l_events, l_det, scale, ret_events);
		});
	}
}

impl<L: Layable> FromIterator<L> for Div<Vec<L>> {
	fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
		let iter = iter.into_iter();
		Div::new(false, iter.collect::<Vec<_>>())
	}
}
