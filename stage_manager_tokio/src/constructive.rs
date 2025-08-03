use std::{cell::RefCell, fmt::Debug, ops::DerefMut, rc::Rc};

use stage_manager::StageChange;
use sui::{DynamicLayable, Layable, LayableExt, core::ReturnEvent};
use tokio::{
	sync::mpsc::{Receiver, error::TryRecvError},
	task::JoinHandle,
};

#[derive(Debug)]
/// returns true if it's finished
pub enum ConstructFunction<T, P> {
	Simple(fn(&mut T, P) -> bool),
	NeedsSuiHandle(fn(&mut T, P, &mut sui::Handle) -> bool),
}

/// A [`loader`](crate::Loader) variation that can send multiple, smaller packets of data to the syncland (P),
/// with a function to put that all into a collective storage (T), and the same post_process function the basic
/// loader has. has many possible uses but maybe most useful for texture loading
pub struct ConstructiveLoader<T, P: Send + 'static, PostProcess: Fn(T) -> StageChange<'static>> {
	pub loading_screen: DynamicLayable<'static>,

	handle: JoinHandle<()>,
	rx: Rc<RefCell<Receiver<P>>>,

	/// used to report from the construct function called from render whether it's finished
	finished: Rc<RefCell<bool>>,

	t: Rc<RefCell<Option<T>>>,
	construct: ConstructFunction<T, P>,
	post_process: PostProcess,
}

impl<T: Debug, P: Send + 'static + Debug, PostProcess: Fn(T) -> StageChange<'static>> Debug
	for ConstructiveLoader<T, P, PostProcess>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ConstructiveLoader")
			.field("loading_screen", &self.loading_screen)
			.field("handle", &self.handle)
			.field("rx", &self.rx)
			.field("finished", &self.finished)
			.field("t", &self.t)
			.field("construct", &self.construct)
			.finish()
	}
}

impl<T, P: Send + 'static, PostProcess: Fn(T) -> StageChange<'static>>
	ConstructiveLoader<T, P, PostProcess>
{
	pub fn new_explicit<F: Future<Output = ()> + Send + 'static>(
		loading_screen: impl Layable + Debug + Clone + 'static,
		f: impl FnOnce(tokio::sync::mpsc::Sender<P>) -> F,
		base_t: T,
		construct: ConstructFunction<T, P>,
		post_process: PostProcess,
	) -> Self {
		let loading_screen = DynamicLayable::new(loading_screen);

		let (tx, rx) = tokio::sync::mpsc::channel(5);

		let f = f(tx);
		let handle = tokio::task::spawn(f);

		let rx = Rc::new(RefCell::new(rx));
		let t = Rc::new(RefCell::new(Some(base_t)));
		let finished = Rc::new(RefCell::new(false));

		Self {
			loading_screen,
			handle,
			rx,
			t,
			construct,
			post_process,
			finished,
		}
	}

	/// creates a new loader and wraps it into a StageChange
	pub fn new_overlay<F: Future<Output = ()> + Send + 'static>(
		overlay: impl Layable + Debug + Clone + 'static,
		f: impl FnOnce(tokio::sync::mpsc::Sender<P>) -> F + 'static,
		base_t: T,
		construct: ConstructFunction<T, P>,
		post_process: PostProcess,
	) -> StageChange<'static>
	where
		T: 'static,
		PostProcess: 'static,
	{
		StageChange::swapper(move |old_stage| {
			let loading_screen = old_stage.overlay(overlay);
			let loader = Self::new_explicit(loading_screen, f, base_t, construct, post_process);

			sui::DynamicLayable::new_notraits(loader)
		})
	}
}

impl<T, P: Send + 'static, PostProcess: Fn(T) -> StageChange<'static>> Layable
	for ConstructiveLoader<T, P, PostProcess>
{
	fn size(&self) -> (i32, i32) {
		self.loading_screen.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.loading_screen.render(d, det, scale);

		match &self.construct {
			ConstructFunction::NeedsSuiHandle(f) => {
				let finished = match self.rx.borrow_mut().try_recv() {
					Ok(item) => {
						let mut t = self.t.borrow_mut();
						if let Some(t) = t.deref_mut() {
							f(t, item, d)
						} else {
							panic!(
								"ConstructiveLoader.t has already been taken, but the render function still got called.\nmaybe you don't have a stage_manager::Stage in the ui stack\n(ConstructFunction::NeedsSuiHandle)"
							)
						}
					}
					Err(TryRecvError::Empty) => false,
					Err(TryRecvError::Disconnected) => {
						panic!(
							"stage_manager_tokio::ConstructiveLoader's receiver disconnected while waiting for results to be yield\nyour construct function probably isn't set up to return true when it's finished\n(ConstructFunction::NeedsSuiHandle)"
						)
					}
				};
				if finished {
					let mut finished = self.finished.borrow_mut();
					*finished = true;
				}
			}
			_ => {}
		}
	}

	fn tick(&mut self) {
		self.loading_screen.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		let finished = match &self.construct {
			ConstructFunction::Simple(f) => match self.rx.borrow_mut().try_recv() {
				Ok(item) => {
					let mut t = self.t.borrow_mut();
					if let Some(t) = t.deref_mut() {
						f(t, item)
					} else {
						panic!(
							"ConstructiveLoader.t has already been taken, but the render function still got called.\nmaybe you don't have a stage_manager::Stage in the ui stack"
						)
					}
				}
				Err(TryRecvError::Empty) => false,
				Err(TryRecvError::Disconnected) => {
					panic!(
						"stage_manager_tokio::ConstructiveLoader's receiver disconnected while waiting for results to be yield\nyour construct function probably isn't set up to return true when it's finished"
					)
				}
			},
			ConstructFunction::NeedsSuiHandle(_) => *self.finished.borrow(),
		};
		let opt = if finished {
			let t = self
				.t
				.borrow_mut()
				.take()
				.expect("ConstructiveLoader finished, but self.t has already been taken");
			let l = (self.post_process)(t);

			Some(ReturnEvent::new(l))
		} else {
			None
		};

		self.loading_screen
			.pass_events(events, det, scale)
			.chain(opt)
	}
}
