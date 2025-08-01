use stage_manager::StageChange;
use std::fmt::Debug;
use sui::{DynamicLayable, Layable, core::ReturnEvent};
use tokio::{
	sync::mpsc::{Receiver, error::TryRecvError},
	task::JoinHandle,
};

// TODO write documentation
#[derive(Debug)]
pub struct Loading<T: Send> {
	loading_screen: DynamicLayable<'static>,

	handle: JoinHandle<()>,
	rx: Receiver<T>,
	post_process: fn(T) -> sui::DynamicLayable<'static>,
}
impl<T: Send + 'static> Loading<T> {
	pub fn new<L: Layable + Debug + Clone + 'static, F: Future<Output = T> + Send + 'static>(
		loading_screen: L,
		future: F,
		post_process: fn(T) -> sui::DynamicLayable<'static>,
	) -> Self {
		Self::from_dyn(DynamicLayable::new(loading_screen), future, post_process)
	}

	pub fn from_dyn<F: Future<Output = T> + Send + 'static>(
		loading_screen: DynamicLayable<'static>,
		future: F,
		post_process: fn(T) -> sui::DynamicLayable<'static>,
	) -> Self {
		let (tx, rx) = tokio::sync::mpsc::channel(1);

		let handle = tokio::task::spawn(async move {
			let output = future.await;
			match tx.send(output).await {
				Ok(a) => a,
				Err(err) => panic!(
					"stage_manager_tokio::Loading: async function finished, couldn't send result into channel\n{err}",
				),
			};
		});

		Self {
			loading_screen,
			handle,
			rx,
			post_process,
		}
	}
}

impl<T: Send> Layable for Loading<T> {
	fn size(&self) -> (i32, i32) {
		self.loading_screen.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.loading_screen.render(d, det, scale);
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		let a = match self.rx.try_recv() {
			Ok(item) => {
				let processed = (self.post_process)(item);
				Some(ReturnEvent::new(StageChange::from_dyn(processed)))
			}
			Err(TryRecvError::Empty) => None,
			Err(TryRecvError::Disconnected) => {
				panic!(
					"stage_manager_tokio::Loading's receiver disconnected before yielding a result"
				)
			}
		};

		self.loading_screen
			.pass_events(events, det, scale)
			.chain(a.into_iter())
	}
}
