use std::ops::DerefMut;

use raylib::{
	ffi::{KeyboardKey, MouseButton},
	RaylibHandle,
};

use crate::{
	core::{Event, FeaturedReturn, ReturnEvent},
	form::FocusHandler,
	Details, Layable,
};

/// `RootContext` contains everything needed to calculate Details and scales, for both rendering
/// and events. this is so there's no way [Layable::render] and [Layable::pass_event]
/// could work with different data.
pub struct RootContext<L: Layable> {
	layable: L,
	det: Details,
	scale: f32,
}
impl<L: Layable> RootContext<L> {
	pub fn new(layable: L, det: Details, scale: f32) -> Self {
		RootContext {
			layable,
			det,
			scale,
		}
	}

	pub fn render(&self, d: &mut crate::Handle) {
		self.layable.render(d, self.det, self.scale);
	}

	pub fn tick(&mut self) {
		self.layable.tick();
	}
	pub fn handle_input<'b, E: FeaturedReturn, H: DerefMut<Target = RaylibHandle>>(
		&'b mut self,
		rl: &mut H,
		focus: &FocusHandler,
	) -> Vec<Result<E, ReturnEvent>> {
		let mut events_to_fire = Vec::new();

		use crate::core::KeyboardEvent;
		use crate::core::MouseEvent;

		//* mouse events

		let (ptr_x, ptr_y) = (rl.get_mouse_x(), rl.get_mouse_y());

		if ptr_x as f32 > self.det.x as f32 && ptr_y as f32 > self.det.y as f32 {
			if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
				events_to_fire.push(Event::MouseEvent(MouseEvent::MouseClick {
					x: ptr_x,
					y: ptr_y,
				}))
			};
			if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
				events_to_fire.push(Event::MouseEvent(MouseEvent::MouseHeld {
					x: ptr_x,
					y: ptr_y,
				}))
			};
			if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
				events_to_fire.push(Event::MouseEvent(MouseEvent::MouseRelease {
					x: ptr_x,
					y: ptr_y,
				}))
			};

			let mouse_wheel_move = rl.get_mouse_wheel_move();
			if mouse_wheel_move != 0.0 {
				events_to_fire.push(Event::MouseEvent(MouseEvent::Scroll {
					x: ptr_x,
					y: ptr_y,
					amount: mouse_wheel_move,
				}))
			};
		};

		let key = rl.get_char_pressed();
		let key = match key {
			Some(a) => Some(a),
			None => {
				if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
					Some(crate::form::typable::BACKSPACE)
				} else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
					Some('\n')
				} else {
					None
				}
			}
		};
		if let Some(key) = key {
			events_to_fire.push(Event::KeyboardEvent(
				focus.get(),
				KeyboardEvent::CharPressed(key),
			))
		}

		let key_downs = keys_to_poll().filter_map(|key| {
			if rl.is_key_down(key) {
				Some(Event::KeyboardEvent(
					focus.get(),
					KeyboardEvent::KeyDown(key),
				))
			} else {
				None
			}
		});
		let events_to_fire = events_to_fire.into_iter().chain(key_downs);

		events_to_fire
			.map(|event| {
				self.layable
					.pass_event(event, self.det, self.scale)
					.map(|event| {
						let cast = E::cast_event(event);
						if cast.can_take::<E>() {
							Ok(cast
								.take()
								.expect("can_take returned true but taking failed"))
						} else {
							Err(cast)
						}
					})
			})
			.filter_map(|a| a)
			.collect::<Vec<_>>()
	}
}

/// every key in [KeyboardKey]
fn keys_to_poll() -> impl Iterator<Item = KeyboardKey> {
	[
		KeyboardKey::KEY_NULL,
		KeyboardKey::KEY_APOSTROPHE,
		KeyboardKey::KEY_COMMA,
		KeyboardKey::KEY_MINUS,
		KeyboardKey::KEY_PERIOD,
		KeyboardKey::KEY_SLASH,
		KeyboardKey::KEY_ZERO,
		KeyboardKey::KEY_ONE,
		KeyboardKey::KEY_TWO,
		KeyboardKey::KEY_THREE,
		KeyboardKey::KEY_FOUR,
		KeyboardKey::KEY_FIVE,
		KeyboardKey::KEY_SIX,
		KeyboardKey::KEY_SEVEN,
		KeyboardKey::KEY_EIGHT,
		KeyboardKey::KEY_NINE,
		KeyboardKey::KEY_SEMICOLON,
		KeyboardKey::KEY_EQUAL,
		KeyboardKey::KEY_A,
		KeyboardKey::KEY_B,
		KeyboardKey::KEY_C,
		KeyboardKey::KEY_D,
		KeyboardKey::KEY_E,
		KeyboardKey::KEY_F,
		KeyboardKey::KEY_G,
		KeyboardKey::KEY_H,
		KeyboardKey::KEY_I,
		KeyboardKey::KEY_J,
		KeyboardKey::KEY_K,
		KeyboardKey::KEY_L,
		KeyboardKey::KEY_M,
		KeyboardKey::KEY_N,
		KeyboardKey::KEY_O,
		KeyboardKey::KEY_P,
		KeyboardKey::KEY_Q,
		KeyboardKey::KEY_R,
		KeyboardKey::KEY_S,
		KeyboardKey::KEY_T,
		KeyboardKey::KEY_U,
		KeyboardKey::KEY_V,
		KeyboardKey::KEY_W,
		KeyboardKey::KEY_X,
		KeyboardKey::KEY_Y,
		KeyboardKey::KEY_Z,
		KeyboardKey::KEY_LEFT_BRACKET,
		KeyboardKey::KEY_BACKSLASH,
		KeyboardKey::KEY_RIGHT_BRACKET,
		KeyboardKey::KEY_GRAVE,
		KeyboardKey::KEY_SPACE,
		KeyboardKey::KEY_ESCAPE,
		KeyboardKey::KEY_ENTER,
		KeyboardKey::KEY_TAB,
		KeyboardKey::KEY_BACKSPACE,
		KeyboardKey::KEY_INSERT,
		KeyboardKey::KEY_DELETE,
		KeyboardKey::KEY_RIGHT,
		KeyboardKey::KEY_LEFT,
		KeyboardKey::KEY_DOWN,
		KeyboardKey::KEY_UP,
		KeyboardKey::KEY_PAGE_UP,
		KeyboardKey::KEY_PAGE_DOWN,
		KeyboardKey::KEY_HOME,
		KeyboardKey::KEY_END,
		KeyboardKey::KEY_CAPS_LOCK,
		KeyboardKey::KEY_SCROLL_LOCK,
		KeyboardKey::KEY_NUM_LOCK,
		KeyboardKey::KEY_PRINT_SCREEN,
		KeyboardKey::KEY_PAUSE,
		KeyboardKey::KEY_F1,
		KeyboardKey::KEY_F2,
		KeyboardKey::KEY_F3,
		KeyboardKey::KEY_F4,
		KeyboardKey::KEY_F5,
		KeyboardKey::KEY_F6,
		KeyboardKey::KEY_F7,
		KeyboardKey::KEY_F8,
		KeyboardKey::KEY_F9,
		KeyboardKey::KEY_F10,
		KeyboardKey::KEY_F11,
		KeyboardKey::KEY_F12,
		KeyboardKey::KEY_LEFT_SHIFT,
		KeyboardKey::KEY_LEFT_CONTROL,
		KeyboardKey::KEY_LEFT_ALT,
		KeyboardKey::KEY_LEFT_SUPER,
		KeyboardKey::KEY_RIGHT_SHIFT,
		KeyboardKey::KEY_RIGHT_CONTROL,
		KeyboardKey::KEY_RIGHT_ALT,
		KeyboardKey::KEY_RIGHT_SUPER,
		KeyboardKey::KEY_KB_MENU,
		KeyboardKey::KEY_KP_0,
		KeyboardKey::KEY_KP_1,
		KeyboardKey::KEY_KP_2,
		KeyboardKey::KEY_KP_3,
		KeyboardKey::KEY_KP_4,
		KeyboardKey::KEY_KP_5,
		KeyboardKey::KEY_KP_6,
		KeyboardKey::KEY_KP_7,
		KeyboardKey::KEY_KP_8,
		KeyboardKey::KEY_KP_9,
		KeyboardKey::KEY_KP_DECIMAL,
		KeyboardKey::KEY_KP_DIVIDE,
		KeyboardKey::KEY_KP_MULTIPLY,
		KeyboardKey::KEY_KP_SUBTRACT,
		KeyboardKey::KEY_KP_ADD,
		KeyboardKey::KEY_KP_ENTER,
		KeyboardKey::KEY_KP_EQUAL,
		KeyboardKey::KEY_BACK,
		KeyboardKey::KEY_MENU,
		KeyboardKey::KEY_VOLUME_UP,
		KeyboardKey::KEY_VOLUME_DOWN,
	]
	.into_iter()
}
