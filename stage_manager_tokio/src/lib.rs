use stage_manager::StageChange;
use std::fmt::Debug;
use sui::{DynamicLayable, Layable, core::ReturnEvent};
use tokio::{
	sync::mpsc::{Receiver, error::TryRecvError},
	task::JoinHandle,
};

mod constructive;
pub use constructive::{ConstructFunction, ConstructiveLoader};

/// Loader is an accessory Layable that lets you execute async code in the background while rendering and ticking
/// a loading screen. when the background process (texture loading, file reading, file picking, etc) is finished, it'll send any
/// type `T` to the post_process function, which has to turn `T` into a Layable for the Stage to change to, synchronously. \
/// requires a tokio runtime to be running globally
#[derive(Debug)]
pub struct Loader<T: Send> {
	loading_screen: DynamicLayable<'static>,

	#[allow(unused)]
	handle: JoinHandle<()>,
	rx: Receiver<T>,
	post_process: fn(T) -> sui::DynamicLayable<'static>,
}
impl<T: Send + 'static> Loader<T> {
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
impl<T: Send + 'static + Debug> Loader<T> {
	pub fn stage_change(self) -> StageChange<'static> {
		StageChange::simple_only_debug(self)
	}
}

impl<T: Send> Layable for Loader<T> {
	fn size(&self) -> (i32, i32) {
		self.loading_screen.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.loading_screen.render(d, det, scale);
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
		let a = match self.rx.try_recv() {
			Ok(item) => {
				let processed = (self.post_process)(item);
				Some(ReturnEvent::new(StageChange::Simple(processed)))
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
