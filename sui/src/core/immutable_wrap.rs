use crate::{core::ReturnEvent, Layable};

/// a wrapper for any Layable that makes it possible to implement Layable
/// for an immutable reference
///
/// &L has implemented Layable up until right now so this is here in case any code
/// anywhere depended on that
#[derive(Clone, Debug)]
pub struct ImmutableWrap<'a, L: Layable>(pub &'a L);
impl<'a, L: Layable> ImmutableWrap<'a, L> {
	pub fn new(reference: &'a L) -> Self {
		Self(reference)
	}
}
impl<'a, L: Layable> Layable for ImmutableWrap<'a, L> {
	fn size(&self) -> (i32, i32) {
		self.0.size()
	}
	fn render(&self, d: &mut super::Handle, det: super::Details, scale: f32) {
		self.0.render(d, det, scale);
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = super::Event>,
		_det: super::Details,
		_scale: f32,
		_ret_events: &mut Vec<ReturnEvent>,
	) {
		println!(
			"dropped all events passed to ImmutableLayable; passing events requires mutability\nevents: ["
		);
		for event in events {
			println!("  {event:?}");
		}
		println!("]");
	}
}
