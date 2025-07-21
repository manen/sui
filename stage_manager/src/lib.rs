use std::{cell::RefCell, fmt::Debug, ops::DerefMut, rc::Rc};

use sui::{DynamicLayable, Layable};

/// a command to have the stage manager change the stage to the dynamic layable within
#[derive(Clone, Debug)]
pub struct StageChange<'a> {
	pub to: DynamicLayable<'a>,
	pub requires_ticking: bool,
}
impl<'a> StageChange<'a> {
	pub fn new<L: Layable + Debug + Clone + 'a>(layable: L) -> Self {
		Self {
			to: DynamicLayable::new(layable),
			requires_ticking: false,
		}
	}
}

#[derive(Clone, Debug)]
/// manages the stage \
/// the point is that it can swap out what it's rendering dynamically at runtime,
/// and any event inside can request a stage change using [StageChange]
pub struct Stage<'a> {
	comp: Rc<RefCell<DynamicLayable<'a>>>,
}
impl<'a> Stage<'a> {
	pub fn new<L: Layable + Debug + Clone + 'a>(base_comp: L) -> Self {
		Self::from_dyn_layable(DynamicLayable::new(base_comp))
	}
	pub fn from_dyn_layable(dyn_layable: DynamicLayable<'a>) -> Self {
		Self {
			comp: Rc::new(RefCell::new(dyn_layable)),
		}
	}
}
impl<'a> Layable for Stage<'a> {
	fn size(&self) -> (i32, i32) {
		self.comp.borrow().size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.comp.borrow().render(d, det, scale)
	}
	fn pass_event(
		&self,
		event: sui::core::Event,
		det: sui::Details,
		scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		let event = self.comp.borrow().pass_event(event, det, scale);
		match event {
			Some(event) if event.can_take::<StageChange>() => {
				let mut change: StageChange =
					event.take().expect("can_take said yes but couldn't take");

				let mut comp = self.comp.borrow_mut();
				std::mem::swap(comp.deref_mut(), &mut change.to); // <- this will swap the current stage and the requested stage, making it so the request
				// and the old stage get dropped at the end of the scope

				None
			}
			a => a,
		}
	}
}
