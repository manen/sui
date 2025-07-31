use raylib::prelude::RaylibDraw;

use crate::{comp::div::DivComponents, core::Event, Details, Layable};

#[macro_export]
macro_rules! single_size {
	($self:expr) => {{
		single_size!(? $self.horizontal)
	}};
	(? $cond:expr) => {{
		let single_size = if $cond {
			#[inline]
			fn single_size<T>(size: (T, T)) -> T {
				size.0
			}
			single_size
		} else {
			#[inline]
			fn single_size<T>(size: (T, T)) -> T {
				size.1
			}
			single_size
		};
		single_size
	}};
}

const DEBUG: bool = false;
const PASS_EVENT_DEBUG: bool = false;

#[derive(Clone, Debug)]
pub struct SpaceBetween<D: DivComponents> {
	components: D,
	horizontal: bool,
}
impl<D: DivComponents> SpaceBetween<D> {
	pub fn new(components: D) -> Self {
		Self {
			components,
			horizontal: false,
		}
	}
	pub fn new_horizontal(components: D) -> Self {
		Self {
			components,
			horizontal: true,
		}
	}

	pub fn calculate_gap(&self, det: Details, scale: f32) -> i32 {
		let single_size = single_size!(self);

		let mut components = -1;
		let mut total_size = 0;
		for comp in self.components.iter_components() {
			total_size += single_size(comp.size());
			components += 1;
		}
		let components = components.max(1);

		let remaining_space = single_size((det.aw, det.ah)) - total_size;
		let gap_scaled = remaining_space as f32 / components as f32 * scale;

		gap_scaled as i32
	}
}
impl<D: DivComponents> Layable for SpaceBetween<D> {
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
		let gap = self.calculate_gap(det, scale);

		let (mut x, mut y) = (det.x, det.y);
		for comp in self.components.iter_components() {
			let l_size = comp.size();

			let (aw, ah) = if !self.horizontal {
				(det.aw, l_size.1)
			} else {
				(l_size.0, det.ah)
			};

			let l_det = Details { x, y, aw, ah };
			if DEBUG {
				d.draw_rectangle_lines(l_det.x, l_det.y, l_det.aw, l_det.ah, crate::Color::WHITE);
			}
			comp.render(d, l_det, scale);

			if !self.horizontal {
				y += (l_size.1 as f32 + gap as f32) as i32;
			} else {
				x += (l_size.0 as f32 + gap as f32) as i32;
			}
		}

		if DEBUG {
			d.draw_rectangle_lines(det.x, det.y, det.aw, det.ah, crate::Color::ORANGE);
		}
	}

	fn tick(&mut self) {
		self.iter_components_mut()
			.into_iter()
			.flatten()
			.for_each(Layable::tick)
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
	) -> impl Iterator<Item = crate::core::ReturnEvent> {
		let event_f = move |event| match event {
			Event::KeyboardEvent(_, _) => {
				for comp in self.components.iter_components_mut().into_iter().flatten() {
					let ret = comp.pass_events(std::iter::once(event), det, scale).next();
					if let Some(ret) = ret {
						return Some(ret);
					}
				}
				None
			}
			Event::MouseEvent(m_event) => {
				if !det.is_inside_tuple(m_event.at()) {
					return None;
				}

				let gap = self.calculate_gap(det, scale);
				let (mut x, mut y) = (det.x, det.y);

				let components = match self.components.iter_components_mut() {
					Some(a) => a,
					None => return None,
				};
				for comp in components {
					let l_size = comp.size();

					let (aw, ah) = if !self.horizontal {
						(det.aw, l_size.1)
					} else {
						(l_size.0, det.ah)
					};

					let l_det = Details { x, y, aw, ah };
					if PASS_EVENT_DEBUG {
						println!(
							"SpaceBetween handling mouse event; {l_det:?}, {:?}",
							m_event.at()
						);
					}
					if l_det.is_inside_tuple(m_event.at()) {
						return comp
							.pass_events(std::iter::once(event), l_det, scale)
							.next();
					}

					if !self.horizontal {
						y += (l_size.1 as f32 * scale) as i32 + gap;
					} else {
						x += (l_size.0 as f32 * scale) as i32 + gap;
					}
				}
				None
			}
		};

		events.filter_map(event_f)
	}
}
