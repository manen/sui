use std::fmt::Debug;

use sui::{DynamicLayable, Layable, core::ReturnEvent};
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

#[derive(Debug, Clone)]
/// this is the only event type that'll get relayed to the remote controller
pub struct RemoteEvent<T>(pub T);

/// a carbon copy of stage_manager::StageChange that can be sent across threads
pub enum RemoteStageChange {
	Simple(DynamicLayable<'static>),
	Swapper(Box<dyn FnOnce(DynamicLayable<'static>) -> DynamicLayable<'static> + Send>),
}
impl Debug for RemoteStageChange {
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

impl RemoteStageChange {
	pub fn simple<L: Layable + Debug + Clone + 'static>(layable: L) -> Self {
		Self::Simple(DynamicLayable::new(layable))
	}
	pub fn simple_only_debug<L: Layable + Debug + 'static>(layable: L) -> Self {
		Self::Simple(DynamicLayable::new_only_debug(layable))
	}

	pub fn swapper<
		F: FnOnce(DynamicLayable<'static>) -> DynamicLayable<'static> + Send + 'static,
	>(
		swap_fn: F,
	) -> Self {
		Self::Swapper(Box::new(swap_fn))
	}
}

#[derive(Debug)]
/// remote-controlled stage. \
/// requires calling .tick to update content
pub struct RemoteStage<T: Send + Debug> {
	current: DynamicLayable<'static>,

	handle: Option<tokio::task::JoinHandle<()>>,

	stage_rx: Receiver<RemoteStageChange>,
	events_tx: Sender<T>,
}
impl<T: Send + Debug> RemoteStage<T> {
	pub fn spawn_new_explicit<F: Future<Output = ()> + Send + 'static>(
		layable: impl Layable + Debug + 'static,
		controller: impl FnOnce(Sender<RemoteStageChange>, Receiver<T>) -> F + Send,
	) -> Self {
		let ((stage_tx, events_rx), mut s) = Self::new_explicit(layable);

		let controller = controller(stage_tx, events_rx);
		let handle = tokio::task::spawn(controller);

		s.handle = Some(handle);
		s
	}
	/// creates a new, remotely controlled stage
	pub fn spawn_new<F: Future<Output = ()> + Send + 'static>(
		controller: impl FnOnce(Sender<RemoteStageChange>, Receiver<T>) -> F + Send,
	) -> Self {
		Self::spawn_new_explicit(sui::comp::Space::new(10, 10), controller)
	}

	pub fn new_explicit(
		layable: impl Layable + Debug + 'static,
	) -> ((Sender<RemoteStageChange>, Receiver<T>), Self) {
		let layable = sui::custom_only_debug(layable);

		let (stage_tx, stage_rx) = tokio::sync::mpsc::channel(5);
		let (events_tx, events_rx) = tokio::sync::mpsc::channel(10);

		let s = Self {
			current: layable,
			handle: None,
			stage_rx,
			events_tx,
		};

		((stage_tx, events_rx), s)
	}
	pub fn new() -> ((Sender<RemoteStageChange>, Receiver<T>), Self) {
		Self::new_explicit(sui::comp::Space::new(10, 10))
	}

	fn try_recv(&mut self) {
		loop {
			match self.stage_rx.try_recv() {
				Ok(RemoteStageChange::Simple(mut new_stage)) => {
					// self.current = new_stage;
					std::mem::swap(&mut self.current, &mut new_stage)
				}
				Ok(RemoteStageChange::Swapper(swap_fn)) => {
					let current_stage = std::mem::replace(
						&mut self.current,
						sui::custom(sui::comp::Space::new(10, 10)), // ! self.current replaced with a dummy stage for this block
					);
					let new_stage = swap_fn(current_stage);

					self.current = new_stage;
				}
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => {
					panic!(
						"RemoteStage's receiver disconnected while waiting for results to be yielded\nyour async environment that yields the stages should probably made to run forever, as dropping the RemoteStage aborts the task anyway"
					)
				}
			}
		}
	}
}
impl<T: Send + Debug> Drop for RemoteStage<T> {
	fn drop(&mut self) {
		if let Some(handle) = &mut self.handle {
			handle.abort();
		}
	}
}

impl<T: Send + Debug + 'static> Layable for RemoteStage<T> {
	fn size(&self) -> (i32, i32) {
		self.current.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.current.render(d, det, scale)
	}

	fn tick(&mut self) {
		self.try_recv();
		self.current.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		self.try_recv();

		let len_before = ret_events.len();
		self.current.pass_events(events, det, scale, ret_events);

		for event in ret_events.drain(len_before..) {
			if event.can_take::<RemoteEvent<T>>() {
				let event: RemoteEvent<T> = event
					.take()
					.expect("can_take said we can take but we can't take");
				let event = event.0;

				match self.events_tx.try_send(event) {
					Ok(a) => a,
					Err(err) => {
						eprintln!(
							"RemoteStage had to drop RemoteEvent since the buffer was full\nerr: {err}"
						)
					}
				}
			}
		}
	}
}
