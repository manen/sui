use std::ops::{Deref, DerefMut};

use raylib::{prelude::RaylibDrawHandle, RaylibThread};

use crate::form::{FocusHandler, UniqueId};

pub struct Handle<'a> {
	d: RaylibDrawHandle<'a>,
	thread: &'a RaylibThread,
	focus: UniqueId,
}
impl<'a> Handle<'a> {
	pub fn new(d: RaylibDrawHandle<'a>, thread: &'a RaylibThread, fh: &FocusHandler) -> Self {
		Self {
			d,
			thread,
			focus: fh.with_borrow(|a| *a),
		}
	}
	pub fn new_unfocused(d: RaylibDrawHandle<'a>, thread: &'a RaylibThread) -> Self {
		Self {
			d,
			thread,
			focus: UniqueId::null(),
		}
	}

	pub fn thread(&self) -> &RaylibThread {
		&self.thread
	}
	pub fn to_parts(&self) -> (&RaylibDrawHandle<'a>, &RaylibThread) {
		(&self.d, &self.thread)
	}
	pub fn to_parts_mut(&mut self) -> (&mut RaylibDrawHandle<'a>, &RaylibThread) {
		(&mut self.d, &self.thread)
	}

	pub fn focus(&self) -> UniqueId {
		self.focus
	}
	pub fn take(self) -> RaylibDrawHandle<'a> {
		self.d
	}
}
impl<'a> Deref for Handle<'a> {
	type Target = RaylibDrawHandle<'a>;

	fn deref(&self) -> &Self::Target {
		&self.d
	}
}
impl<'a> DerefMut for Handle<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.d
	}
}
