use std::{cell::RefCell, fmt::Debug, ops::DerefMut, rc::Rc};

use sui::{DynamicLayable, Layable, core::ReturnEvent};

pub enum StageChange<'a> {
	Simple(DynamicLayable<'a>),
	Swapper(Box<dyn FnOnce(DynamicLayable<'a>) -> DynamicLayable<'a>>),
}
impl<'a> Debug for StageChange<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Simple(res) => f.debug_tuple("StageChange::Simple").field(&res).finish(),
			Self::Swapper(_) => f
				.debug_tuple("StageChange::Swapper")
				.field(&"swap_fn")
				.finish(),
		}
	}
}

impl<'a> StageChange<'a> {
	pub fn simple<L: Layable + Debug + Clone + 'a>(layable: L) -> Self {
		Self::Simple(DynamicLayable::new(layable))
	}
	pub fn simple_only_debug<L: Layable + Debug + 'a>(layable: L) -> Self {
		Self::Simple(DynamicLayable::new_only_debug(layable))
	}

	pub fn swapper<F: FnOnce(DynamicLayable<'a>) -> DynamicLayable<'a> + 'static>(
		swap_fn: F,
	) -> Self {
		Self::Swapper(Box::new(swap_fn))
	}
}

#[derive(Clone, Debug)]
/// manages the stage \
/// the point is that it can swap out what it's rendering dynamically at runtime,
/// and any event inside can request a stage change using [StageChange]
pub struct Stage {
	comp: Rc<RefCell<DynamicLayable<'static>>>,
}
impl Stage {
	pub fn new<L: Layable + Debug + Clone + 'static>(base_comp: L) -> Self {
		Self::from_dyn_layable(DynamicLayable::new(base_comp))
	}
	pub fn new_only_debug<L: Layable + Debug + 'static>(base_comp: L) -> Self {
		Self::from_dyn_layable(DynamicLayable::new_only_debug(base_comp))
	}
	pub fn from_dyn_layable(dyn_layable: DynamicLayable<'static>) -> Self {
		Self {
			comp: Rc::new(RefCell::new(dyn_layable)),
		}
	}

	/// if u have self.comp already borrowed then you're crashing
	fn handle_rets(&mut self, ret: &mut Vec<ReturnEvent>, len_before: usize) {
		let changed_rng = len_before..ret.len();
		let changed = &ret[changed_rng];

		let stage_change_i = changed
			.iter()
			.enumerate()
			.filter(|(_, ret)| ret.can_take::<StageChange>())
			.map(|(a, _)| a)
			.collect::<Vec<_>>();
		let mut stage_changes = Vec::with_capacity(stage_change_i.len());
		for i in stage_change_i {
			stage_changes.push(ret.swap_remove(i))
		}

		for ret in stage_changes {
			if ret.can_take::<StageChange>() {
				let change: StageChange = ret.take().expect("can_take said yes but couldn't take");

				match change {
					StageChange::Simple(mut new_stage) => {
						let mut comp = self.comp.borrow_mut();
						std::mem::swap(comp.deref_mut(), &mut new_stage); // <- this will swap the current stage and the requested stage, making it so the request
						// and the old stage get dropped at the end of the scope
					}
					StageChange::Swapper(swap_fn) => {
						let mut comp = self.comp.borrow_mut();

						#[allow(unused_unsafe)]
						unsafe {
							let taken_comp = std::mem::replace(
								comp.deref_mut(),
								DynamicLayable::new(sui::comp::Space::new(10, 10)),
							); // ! <- self.comp replaced with a dummy layable for the remainder of the unsafe block!
							let new_stage = swap_fn(taken_comp);

							*comp = new_stage;
						}
					}
				}
			}
		}
	}
}
impl Layable for Stage {
	fn size(&self) -> (i32, i32) {
		self.comp.borrow().size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.comp.borrow().render(d, det, scale)
	}

	fn tick(&mut self) {
		self.comp.borrow_mut().tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let len_before = ret_events.len();
		self.comp
			.borrow_mut()
			.pass_events(events, det, scale, ret_events);

		self.handle_rets(ret_events, len_before);
	}
}
