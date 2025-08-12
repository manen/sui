use std::borrow::Cow;

use crate::{
	comp::{
		self,
		scrollable::{ScrollableMode, ScrollableState},
		Comp, Compatible,
	},
	core::{Event, ImmutableWrap, ReturnEvent},
	Details, DynamicLayable, Layable,
};

pub use crate::root_ctx::*;

pub fn custom<'a, L: Layable + std::fmt::Debug + Clone + 'a>(layable: L) -> DynamicLayable<'a> {
	crate::DynamicLayable::new(layable)
}
pub fn custom_only_debug<'a, L: Layable + std::fmt::Debug + 'a>(layable: L) -> DynamicLayable<'a> {
	crate::DynamicLayable::new_only_debug(layable)
}

pub fn div<D: comp::div::DivComponents>(components: D) -> comp::Div<D> {
	comp::Div::new(false, components)
}
pub fn div_h<D: comp::div::DivComponents>(components: D) -> comp::Div<D> {
	comp::Div::new(true, components)
}
pub fn text<'a, T: Into<Cow<'a, str>>>(text: T, size: i32) -> Comp<'a> {
	comp::Text::new(text, size).into_comp()
}

/// LayableExt provides associated functions for most comp::*::new calls
pub trait LayableExt: Layable + Sized {
	/// see [comp::Centered]
	fn centered(self) -> comp::Centered<Self> {
		comp::Centered::new(self)
	}
	/// see [comp::CenterX]
	fn center_x(self) -> comp::CenterX<Self> {
		comp::CenterX::new(self)
	}
	/// see [comp::CenterY]
	fn center_y(self) -> comp::CenterY<Self> {
		comp::CenterY::new(self)
	}

	/// see [comp::AtEnd]
	fn to_right(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_right(self)
	}
	/// see [comp::AtEnd]
	fn to_bottom(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_bottom(self)
	}
	/// see [comp::AtEnd]
	fn to_bottom_right(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_bottom_right(self)
	}

	/// see [comp::Crop]
	fn crop(self) -> comp::Crop<Self> {
		comp::Crop::new(self)
	}
	/// see [comp::RestrictToSize]
	fn restrict_to_size(self) -> comp::RestrictToSize<Self> {
		comp::RestrictToSize::new(self)
	}

	/// see [comp::FixedSize]
	fn fix_w(self, width: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_w(width, self)
	}
	/// see [comp::FixedSize]
	fn fix_h(self, height: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_h(height, self)
	}
	/// see [comp::FixedSize]
	fn fix_wh(self, width: i32, height: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_size((width, height), self)
	}
	/// see [comp::FixedSize]
	fn fix_wh_square(self, both: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_both(both, self)
	}

	/// see [comp::ScaleToFit]
	fn scale_h_to_fix(self, fix_width: i32) -> comp::ScaleToFit<Self> {
		comp::ScaleToFit::fix_w(fix_width, self)
	}
	/// see [comp::ScaleToFit]
	fn scale_w_to_fix(self, fix_height: i32) -> comp::ScaleToFit<Self> {
		comp::ScaleToFit::fix_h(fix_height, self)
	}

	/// see [comp::Scale]
	fn scale(self, scale: f32) -> comp::Scale<Self> {
		comp::Scale::new(self, scale)
	}

	/// see [comp::Margin]
	fn margin(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::all(margin, self)
	}
	/// see [comp::Margin]
	fn margin_v(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::vertical(margin, self)
	}
	/// see [comp::Margin]
	fn margin_h(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::horizontal(margin, self)
	}

	/// see [comp::View]
	fn view(self, x: i32, y: i32) -> comp::View<Self> {
		comp::View::new(self, x, y)
	}

	/// see [comp::Scrollable]
	fn scrollable_vert(self, state: ScrollableState) -> comp::Scrollable<comp::Crop<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Vertical, self)
	}
	/// see [comp::Scrollable]
	fn scrollable_horiz(self, state: ScrollableState) -> comp::Scrollable<comp::Crop<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Horizontal, self)
	}
	/// see [comp::Scrollable]
	fn scrollable(self, state: ScrollableState) -> comp::Scrollable<comp::Crop<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Both, self)
	}

	/// see [comp::Clickable]
	fn clickable<T: 'static, F: FnMut((i32, i32)) -> T>(
		self,
		gen_ret: F,
	) -> comp::Clickable<Self, F, T> {
		comp::Clickable::new(gen_ret, self)
	}
	/// see [comp::Clickable]
	fn clickable_fallback<T: Clone + 'static, F: FnMut((i32, i32)) -> T>(
		self,
		gen_ret: F,
	) -> comp::Clickable<Self, F, T> {
		comp::Clickable::new_fallback(gen_ret, self)
	}

	/// see [comp::Debug]
	fn debug(self) -> comp::Debug<Self> {
		comp::Debug::new(self)
	}

	/// see [comp::Overlay]
	fn overlay<L1: Layable>(self, foreground: L1) -> comp::Overlay<L1, Self> {
		comp::Overlay::new(self, foreground)
	}
	/// see [comp::Overlay]
	fn with_background<L1: Layable>(self, background: L1) -> comp::Overlay<Self, L1> {
		comp::Overlay::new(background, self)
	}

	/// makes it so you can implement Layable for &L \
	/// the tradeoff is losing pass_event functionality
	fn immutable_wrap(&self) -> ImmutableWrap<Self> {
		ImmutableWrap::new(self)
	}
	/// the context from which root components should be interacted with \
	/// see [RootContext]
	fn root_context(&mut self, det: Details, scale: f32) -> RootContext<&mut Self> {
		RootContext::new(self, det, scale)
	}

	fn pass_events_simple(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
	) -> Vec<ReturnEvent> {
		let mut ret = Vec::new();
		self.pass_events(events, det, scale, &mut ret);
		ret
	}
}
impl<L: Layable> LayableExt for L {}
