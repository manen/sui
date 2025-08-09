use raylib::text::WeakFont;
use std::{
	mem::ManuallyDrop,
	ops::{Deref, DerefMut},
	sync::{Arc, Mutex},
};

static DEFAULT_FONT: Mutex<Option<Font>> = Mutex::new(None);

#[derive(Debug)]
pub struct FontInner {
	pub font: ManuallyDrop<raylib::text::Font>,
	pub needs_drop: bool,
}
impl Drop for FontInner {
	fn drop(&mut self) {
		if self.needs_drop {
			unsafe { ManuallyDrop::drop(&mut self.font) }
		}
	}
}

/// it's a font idk what else to tell you \
///
/// ## Send Safety
/// Font is Send, but don't really use anything that calls raylib font functions
/// from another thread, as that could cause some shit (i haven't seen what) to happen \
///
/// ### what is safe:
/// initializing a Font on another thread (to create a comp::Text, etc), and passing it
/// to the main thread to render
#[derive(Debug, Clone)]
pub struct Font {
	font: Arc<FontInner>,
}
unsafe impl Send for Font {}
impl Default for Font {
	fn default() -> Self {
		let mut lock = match DEFAULT_FONT.lock() {
			Ok(a) => a,
			Err(err) => err.into_inner(),
		};

		// if exists return it, if not, set the default raylib font as the default font
		// and return that
		let font = match lock.deref() {
			Some(a) => a.clone(),
			None => {
				let default_raylib = Font::default_raylib();
				*lock = Some(default_raylib.clone());
				default_raylib
			}
		};

		font
	}
}
impl Font {
	pub fn from_raylib(font: raylib::text::Font, needs_drop: bool) -> Self {
		let font = ManuallyDrop::new(font);
		let font = FontInner { font, needs_drop };
		Self {
			font: Arc::new(font),
		}
	}
	/// needs to be called from the main thread
	pub fn default_raylib() -> Self {
		let font_weak = unsafe { raylib::ffi::GetFontDefault() };
		let font = unsafe { raylib::text::Font::from_raw(font_weak) };

		Self::from_raylib(font, false)
	}

	/// sets this font to be the global default font. use `Font::default()` to get the global default font
	pub fn set_as_global(&self) {
		let mut lock = DEFAULT_FONT
			.lock()
			.expect("failed to lock static DEFAULT_FONT");

		*lock = Some(self.clone())
	}

	pub fn with_font<R, F: FnOnce(&raylib::text::Font) -> R>(&self, f: F) -> R {
		let inner = self.font.as_ref();
		let r = inner.font.deref();

		f(r)
	}
}
