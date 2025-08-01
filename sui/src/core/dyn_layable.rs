use crate::core::{Details, Event, Layable};
use std::fmt::Debug;

/// DynamicLayable is like dyn Layable but better
pub struct DynamicLayable<'a> {
	/// heap pointer, allocated with std::alloc
	ptr: *mut u8,
	layout: std::alloc::Layout,
	type_name: &'static str,

	size: fn(*const u8) -> (i32, i32),
	render: fn(*const u8, d: &mut crate::Handle, det: Details, scale: f32),

	tick: fn(*mut u8),
	pass_events:
		fn(*mut u8, events: Vec<Event>, det: Details, scale: f32) -> Vec<crate::core::ReturnEvent>,

	drop: fn(*mut u8),
	clone: Option<fn(*const u8, std::alloc::Layout) -> *mut u8>,
	debug: Option<fn(*const u8, fmt: &mut std::fmt::Formatter) -> std::fmt::Result>,

	lifetime: std::marker::PhantomData<&'a ()>,
}
// memory stuff for DynamicLayable
impl<'a> DynamicLayable<'a> {
	pub fn new<L: Layable + Debug + Clone + 'a>(layable: L) -> Self {
		Self::new_notraits(layable)
			.add_debug::<L>()
			.add_clone::<L>()
	}
	pub fn new_only_debug<L: Layable + Debug + 'a>(layable: L) -> Self {
		Self::new_notraits(layable).add_debug::<L>()
	}
	pub fn new_only_clone<L: Layable + Clone + 'a>(layable: L) -> Self {
		Self::new_notraits(layable).add_clone::<L>()
	}

	// common trait impls: might cause some really ugly stuff to happen if L != the L new_notraits was called with
	fn add_debug<L: Layable + Debug>(mut self) -> Self {
		// no pretty printed version cause what do u even need that for
		fn debug<L: Layable + Debug>(
			ptr: *const u8,
			fmt: &mut std::fmt::Formatter,
		) -> std::fmt::Result {
			let b: &L = unsafe { &*(ptr as *const L) };
			b.fmt(fmt)
		}

		self.debug = Some(debug::<L>);
		self
	}
	fn add_clone<L: Layable + Clone>(mut self) -> Self {
		/// clone allocates a new pointer for layout and copies layout.size() bytes from ptr into it, returning the new ptr \
		/// things might get really ugly if layout != self.layout, manual memory management is scary
		fn clone<L: Layable + Clone>(ptr: *const u8, layout: std::alloc::Layout) -> *mut u8 {
			let b: &L = unsafe { &*(ptr as *const L) };
			let clone = L::clone(b);

			let new_ptr = unsafe { std::alloc::alloc(layout) };
			unsafe {
				std::ptr::copy_nonoverlapping(
					&clone as *const L as *const u8,
					new_ptr,
					layout.size(),
				)
			};
			std::mem::forget(clone);

			new_ptr
		}

		self.clone = Some(clone::<L>);
		self
	}

	pub fn new_notraits<L: Layable + 'a>(layable: L) -> Self {
		let type_name = std::any::type_name::<L>();
		let layout = std::alloc::Layout::new::<L>();
		let ptr = unsafe { std::alloc::alloc(layout) } as *mut L;
		// copy contents of layable into ptr
		unsafe { std::ptr::copy_nonoverlapping(&layable as *const L, ptr, 1) };
		std::mem::forget(layable);
		let ptr = ptr as *mut u8;

		fn size<L: Layable>(ptr: *const u8) -> (i32, i32) {
			L::size(unsafe { &*(ptr as *const L) })
		}
		fn render<L: Layable>(ptr: *const u8, d: &mut crate::Handle, det: Details, scale: f32) {
			L::render(unsafe { &*(ptr as *const L) }, d, det, scale)
		}
		fn tick<L: Layable>(ptr: *mut u8) {
			L::tick(unsafe { &mut *(ptr as *mut L) });
		}
		fn pass_events<L: Layable>(
			ptr: *mut u8,
			events: Vec<Event>,
			det: Details,
			scale: f32,
		) -> Vec<crate::core::ReturnEvent> {
			L::pass_events(
				unsafe { &mut *(ptr as *mut L) },
				events.into_iter(),
				det,
				scale,
			)
			.collect()
		}

		fn drop<L: Layable>(ptr: *mut u8) {
			let mut layable: std::mem::MaybeUninit<L> = std::mem::MaybeUninit::uninit();
			unsafe { std::ptr::copy_nonoverlapping(ptr as *const L, layable.as_mut_ptr(), 1) };
			unsafe { layable.assume_init_drop() };
		}

		let d = Self {
			ptr,
			layout,
			type_name,
			size: size::<L>,
			render: render::<L>,
			tick: tick::<L>,
			pass_events: pass_events::<L>,
			drop: drop::<L>,
			clone: None,
			debug: None,
			lifetime: std::marker::PhantomData,
		};
		d.null_check();
		d
	}

	/// null_check panics if self.ptr is null and that's it
	fn null_check(&self) {
		if self.ptr as *const _ == std::ptr::null() {
			panic!(
				"DynamicLayable for type {} ended up having null as self.ptr, halting execution",
				self.type_name
			)
		}
	}

	/// borrows self as L, if L is the type inside
	pub fn borrow<L: Layable>(&self) -> Option<&L> {
		self.null_check();
		if self.can_take::<L>() {
			let b = unsafe { &*(self.ptr as *mut L) };
			Some(b)
		} else {
			None
		}
	}
	/// borrows self as L, if L is the type inside
	pub fn borrow_mut<L: Layable>(&mut self) -> Option<&'a mut L> {
		self.null_check();
		if self.can_take::<L>() {
			let b = unsafe { &mut *(self.ptr as *mut L) };
			Some(b)
		} else {
			None
		}
	}
	/// basically [Self::new] but backwards
	pub fn take<L: Layable>(mut self) -> Option<L> {
		self.null_check();
		if self.can_take::<L>() {
			let mut layable: std::mem::MaybeUninit<L> = std::mem::MaybeUninit::uninit();

			unsafe { std::ptr::copy_nonoverlapping(self.ptr as *const L, layable.as_mut_ptr(), 1) };

			// double free protection
			// with this, dropping self at the end of this function will only deallocate the pointer
			fn empty_drop(_: *mut u8) {}
			self.drop = empty_drop;

			let layable = unsafe { layable.assume_init() };
			Some(layable)
		} else {
			None
		}
	}
	/// returns whether calling self.take<L> will return Some \
	/// only here because Self::take takes by value not by reference
	pub fn can_take<L: Layable>(&self) -> bool {
		// this is our bulletproof type-checking
		std::any::type_name::<L>() == self.type_name
			&& self.layout == std::alloc::Layout::new::<L>()
	}
}
impl<'a> Drop for DynamicLayable<'a> {
	fn drop(&mut self) {
		(self.drop)(self.ptr);
		unsafe { std::alloc::dealloc(self.ptr as *mut u8, self.layout) };
	}
}
// layable impl
impl<'a> Layable for DynamicLayable<'a> {
	fn size(&self) -> (i32, i32) {
		(self.size)(self.ptr)
	}
	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		(self.render)(self.ptr, d, det, scale)
	}

	fn tick(&mut self) {
		(self.tick)(self.ptr)
	}

	/// pass_events works with vecs, so allocations unfortunately \
	/// the more events there less efficiency loss this is, so while playing and pressing and holding many keys
	/// this might actually be better than calling each event individually
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
	) -> impl Iterator<Item = crate::core::ReturnEvent> {
		(self.pass_events)(self.ptr, events.collect(), det, scale).into_iter()
	}
}
// common trait impls
impl<'a> Debug for DynamicLayable<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.null_check();
		match self.debug {
			None => write!(f, "[DynamicLayable {}]", self.type_name),
			Some(dbgf) => {
				fn none_or_some<T>(x: Option<T>) -> &'static str {
					match x {
						Some(_) => "Some",
						None => "None",
					}
				}
				let (clone, debug) = (none_or_some(self.clone), none_or_some(self.debug));

				write!(f, "[DynamicLayable ")?;
				dbgf(self.ptr, f)?;
				write!(f, ", clone: {clone}, debug: {debug}]")
			}
		}
	}
}
impl<'a> Clone for DynamicLayable<'a> {
	fn clone(&self) -> Self {
		self.null_check();
		match self.clone {
			None => panic!("attempted to clone a DynamicLayable that didn't implement cloning\nmake sure to use DynamicLayable::new or DynamicLayable::new_only_clone\nsorry for panicking but the only other option is memory corruption so i think u still got a good deal"),
			Some(clonef) => {
				let new_ptr = clonef(self.ptr, self.layout);

				let cloned = Self {
					ptr: new_ptr,
					layout: self.layout,
					type_name: self.type_name,
					size: self.size,
					render: self.render,
					tick: self.tick,
					pass_events: self.pass_events,
					drop: self.drop,
					clone: self.clone,
					debug: self.debug,
					lifetime: std::marker::PhantomData,
				};
				cloned.null_check();
				cloned
			}
		}
	}
}

#[cfg(test)]
mod dynamiclayable_tests {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq)]
	struct TakeDummy(Vec<i32>);
	impl Layable for TakeDummy {
		fn size(&self) -> (i32, i32) {
			(200, 200)
		}
		fn render(&self, _: &mut crate::Handle, _: Details, _: f32) {}
		fn pass_events(
			&mut self,
			events: impl Iterator<Item = Event>,
			_: Details,
			_: f32,
		) -> impl Iterator<Item = crate::core::ReturnEvent> {
			events.map(Event::ret)
		}
	}

	#[test]
	fn test_assert() {
		eprintln!("begin assert testing DynamicLayable");
		test_single(TakeDummy(Vec::new()));
		test_single(crate::div(vec![
			TakeDummy(Vec::new()),
			TakeDummy(Vec::new()),
			TakeDummy(Vec::new()),
		]));
		test_single(crate::comp::Div::new(
			false,
			vec![
				TakeDummy(Vec::new()),
				TakeDummy(Vec::new()),
				TakeDummy(Vec::new()),
			],
		));
	}
	fn test_single<L: Layable + Clone + Debug>(l: L) {
		let d = DynamicLayable::new(l.clone());
		println!("{d:?}");
		test_pair(l, d);
	}
	fn test_pair<A: Layable, B: Layable>(mut a: A, mut b: B) {
		let test_events = [Event::MouseEvent(
			crate::core::event::MouseEvent::MouseClick { x: 3, y: 4 },
		)]
		.into_iter();

		assert_eq!(a.size(), b.size());
		assert_eq!(
			a.pass_events(test_events.clone(), Default::default(), 1.0)
				.next()
				.map(|ret| ret.take::<Event>()),
			b.pass_events(test_events, Default::default(), 1.0)
				.next()
				.map(|ret| ret.take())
		);
	}

	#[test]
	fn test_clone() {
		let d_a = DynamicLayable::new(TakeDummy(vec![4, 5, 6]));
		let d_b = d_a.clone();

		test_pair(d_a, d_b);

		let mut inc = 0..;
		let mut gen_inc = move || (&mut inc).take(3).collect::<Vec<_>>();

		let d_c = DynamicLayable::new(crate::div(vec![
			TakeDummy(gen_inc()),
			TakeDummy(gen_inc()),
			TakeDummy(gen_inc()),
		]));
		let d_d = d_c.clone();

		test_pair(d_c, d_d);
	}

	static mut DROPPED: bool = false;
	#[test]
	fn test_drop() {
		#[derive(Clone, Debug)]
		struct Dummy;
		impl Layable for Dummy {
			fn size(&self) -> (i32, i32) {
				(200, 200)
			}
			fn render(&self, _: &mut crate::Handle, _: Details, _: f32) {}
			fn pass_events(
				&mut self,
				events: impl Iterator<Item = Event>,
				_: Details,
				_: f32,
			) -> impl Iterator<Item = crate::core::ReturnEvent> {
				events.map(Event::ret)
			}
		}
		impl Drop for Dummy {
			fn drop(&mut self) {
				unsafe { DROPPED = true };
			}
		}

		{
			let _ = DynamicLayable::new(Dummy);
		}
		assert!(unsafe { DROPPED });
	}

	#[test]
	fn test_take() {
		let d = DynamicLayable::new(TakeDummy(vec![30]));
		let d_cloned = d.clone();

		assert!(!d_cloned.can_take::<crate::Comp>());
		assert!(!d_cloned.can_take::<crate::Text>());
		assert!(!d.can_take::<crate::Comp>());
		assert!(!d.can_take::<crate::Text>());

		assert_eq!(d_cloned.take(), Some(TakeDummy(vec![30])));
		assert_eq!(d.take(), Some(TakeDummy(vec![30])));
	}

	#[test]
	fn test_borrow() {
		#[derive(Clone, Debug, PartialEq, Eq)]
		struct Dummy(i32);
		impl Layable for Dummy {
			fn size(&self) -> (i32, i32) {
				(200, 200)
			}
			fn render(&self, _: &mut crate::Handle, _: Details, _: f32) {}
			fn pass_events(
				&mut self,
				events: impl Iterator<Item = Event>,
				_: Details,
				_: f32,
			) -> impl Iterator<Item = crate::core::ReturnEvent> {
				events.map(Event::ret)
			}
		}

		let mut d = DynamicLayable::new(Dummy(10));

		assert!(d.borrow::<crate::Div<Dummy>>().is_none());
		match d.borrow_mut::<crate::Text>() {
			None => { /* good!! */ }
			Some(_) => panic!("d.borrow_mut for the incorrect type returned something?"),
		}

		assert_eq!(d.borrow::<Dummy>(), Some(&Dummy(10)));

		d.borrow_mut::<Dummy>().expect("this is the correct type").0 = 3;

		assert_eq!(d.borrow::<Dummy>(), Some(&Dummy(3)));
		assert_eq!(d.take::<Dummy>(), Some(Dummy(3)))
	}

	// #[test]
	// fn this_should_not_compile() {
	// 	let d = {
	// 		let s = "hello this data is going to disappear real soon".to_owned();
	// 		DynamicLayable::new(crate::text(&s, 13)) // <- s does not live long enough
	// 	};
	// 	d.size();

	// 	let b = {
	// 		let s = "this is just a string!!".to_owned();
	// 		let d = DynamicLayable::new(crate::text(s, 4));
	// 		d.borrow::<crate::Comp>().expect("this is the correct type") // <- d does not live long enough
	// 	};
	// 	b.size();
	// }
}
