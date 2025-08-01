use std::{cell::RefCell, fmt::Debug, ops::DerefMut, rc::Rc};

use sui::{DynamicLayable, Layable, core::ReturnEvent};

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
	pub fn from_dyn(dyn_layable: DynamicLayable<'a>) -> Self {
		Self {
			to: dyn_layable,
			requires_ticking: false,
		}
	}
	pub fn from_dyn_ticking(dyn_layable: DynamicLayable<'a>) -> Self {
		Self {
			to: dyn_layable,
			requires_ticking: true,
		}
	}
}

#[derive(Clone, Debug)]
/// manages the stage \
/// the point is that it can swap out what it's rendering dynamically at runtime,
/// and any event inside can request a stage change using [StageChange]
pub struct Stage<'a> {
	comp: Rc<RefCell<DynamicLayable<'a>>>,
	ticking: bool,
}
impl<'a> Stage<'a> {
	pub fn new<L: Layable + Debug + Clone + 'a>(base_comp: L) -> Self {
		Self::from_dyn_layable(DynamicLayable::new(base_comp))
	}
	pub fn from_dyn_layable(dyn_layable: DynamicLayable<'a>) -> Self {
		Self {
			comp: Rc::new(RefCell::new(dyn_layable)),
			ticking: false,
		}
	}

	/// if u have self.comp already borrowed then you're crashing
	fn handle_rets(&mut self, ret: Vec<ReturnEvent>) -> Vec<ReturnEvent> {
		let mut ret_back = Vec::with_capacity(ret.len());
		for ret in ret {
			if ret.can_take::<StageChange>() {
				let mut change: StageChange =
					ret.take().expect("can_take said yes but couldn't take");

				let mut comp = self.comp.borrow_mut();
				std::mem::swap(comp.deref_mut(), &mut change.to); // <- this will swap the current stage and the requested stage, making it so the request
				// and the old stage get dropped at the end of the scope

				self.ticking = change.requires_ticking;
			} else {
				ret_back.push(ret);
			}
		}
		ret_back
	}
}
impl<'a> Layable for Stage<'a> {
	fn size(&self) -> (i32, i32) {
		self.comp.borrow().size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.comp.borrow().render(d, det, scale)
	}

	fn tick(&mut self) {
		self.comp.borrow_mut().tick();

		if self.ticking {
			let ret = self
				.comp
				.borrow_mut()
				.pass_events(std::iter::empty(), Default::default(), 1.0)
				.collect::<Vec<_>>();

			self.handle_rets(ret);
		}
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		let ret = self
			.comp
			.borrow_mut()
			.pass_events(events, det, scale)
			.collect::<Vec<_>>(); // too many allocs this hurt to write

		self.handle_rets(ret).into_iter()
	}
}
