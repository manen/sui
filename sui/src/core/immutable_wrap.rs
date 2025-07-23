use crate::Layable;

/// a wrapper for any Layable that makes it possible to implement Layable
/// for an immutable reference
///
/// &L has implemented Layable up until right now so this is here in case any code
/// anywhere depended on that
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
	fn pass_event(
		&mut self,
		event: super::Event,
		_det: super::Details,
		_scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		println!(
			"dropped {event:?} passed to ImmutableLayable; passing events requires mutability"
		);
		None
	}
}
