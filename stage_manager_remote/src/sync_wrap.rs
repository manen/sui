use stage_manager::StageChange;
use sui::{Layable, core::ReturnEvent};
use tokio::sync::mpsc;

use crate::RemoteStageChange;

#[derive(Debug)]
/// lets you create a channel for RemoteStageChanges, that'll turn it into
/// regular StageChanges and send them up the ui stack, swapping a regular `stage_manager`
/// scene remotely
pub struct StageSyncWrap<L: Layable> {
	layable: L,
	rx: mpsc::Receiver<RemoteStageChange>,
}
impl<L: Layable> StageSyncWrap<L> {
	pub fn new(layable: L) -> (Self, mpsc::Sender<RemoteStageChange>) {
		let (tx, rx) = mpsc::channel(5);
		let sync_wrap = Self { layable, rx };

		(sync_wrap, tx)
	}
	pub fn assemble(layable: L, rx: mpsc::Receiver<RemoteStageChange>) -> Self {
		Self { layable, rx }
	}
}
impl<L: Layable> Layable for StageSyncWrap<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.layable.render(d, det, scale);
	}

	fn tick(&mut self) {
		self.layable.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		self.layable.pass_events(events, det, scale, ret_events);

		loop {
			match self.rx.try_recv() {
				Ok(remote_stage_change) => {
					let stage_change = match remote_stage_change {
						RemoteStageChange::Simple(comp) => StageChange::Simple(comp),
						RemoteStageChange::Swapper(swap_fn) => StageChange::Swapper(swap_fn),
					};
					ret_events.push(ReturnEvent::new(stage_change))
				}
				Err(_) => break,
			}
		}
	}
}
