use crate::{Details, Layable};

use raylib::{
	color::Color,
	math::{Rectangle, Vector2},
	prelude::RaylibDraw,
	texture::{RaylibTexture2D, Texture2D},
};

pub fn render_to_raylib_tex<L: Layable>(
	layable: &L,
	d: &mut crate::Handle,
	det: Details,
	scale: f32,
) -> Texture2D {
	let target = unsafe {
		raylib::ffi::LoadRenderTexture(
			(det.aw as f32 * scale) as i32,
			(det.ah as f32 * scale) as i32,
		)
	};

	// whole lotta unsafe but safe raylib crate is stupid
	{
		unsafe {
			raylib::ffi::BeginTextureMode(target);
		};

		d.clear_background(Color::new(0, 0, 0, 0));
		layable.render(d, Details { x: 0, y: 0, ..det }, scale);

		unsafe {
			raylib::ffi::EndTextureMode();
		}
	}

	let tex = target.texture;
	let tex = unsafe { Texture2D::from_raw(tex) };

	let mut image = tex
		.load_image()
		.expect("tex.load_image failed in render_to_raylib_tex");
	image.flip_vertical();

	let flipped_tex = unsafe { raylib::ffi::LoadTextureFromImage(image.to_raw()) };

	unsafe { Texture2D::from_raw(flipped_tex) }
}

#[derive(Debug)]
pub struct Texture {
	tex: Texture2D,
}
impl Clone for Texture {
	fn clone(&self) -> Self {
		let image = self
			.tex
			.load_image()
			.expect("Texture.tex.load_image failed");
		let image = unsafe { image.unwrap() };
		let new_tex = unsafe { raylib::ffi::LoadTextureFromImage(image) };
		let new_tex = unsafe { Texture2D::from_raw(new_tex) };

		Self { tex: new_tex }
	}
}
impl AsRef<Texture2D> for Texture {
	fn as_ref(&self) -> &Texture2D {
		&self.tex
	}
}
impl Texture {
	pub fn new_from_raylib(tex: Texture2D) -> Self {
		Self { tex }
	}
	pub fn from_layable<L: Layable>(d: &mut crate::Handle, layable: &L) -> Self {
		let (w, h) = layable.size();
		let tex = render_to_raylib_tex(
			layable,
			d,
			Details {
				aw: w,
				ah: h,
				..Default::default()
			},
			1.0,
		);
		Self::new_from_raylib(tex)
	}

	pub fn size(&self) -> (i32, i32) {
		(self.tex.width, self.tex.height)
	}
	pub fn width(&self) -> i32 {
		self.tex.width
	}
	pub fn height(&self) -> i32 {
		self.tex.height
	}

	pub fn render(&self, d: &mut crate::Handle, det: Details) {
		self.render_with_rotation(d, det, 0.0);
	}
	/// does not correct for the position change caused by the rotation
	pub fn render_with_rotation(&self, d: &mut crate::Handle, det: Details, degrees: f32) {
		d.draw_texture_pro(
			&self.tex,
			Rectangle {
				x: 0.0,
				y: 0.0,
				width: self.tex.width as _,
				height: self.tex.height as _,
			},
			Rectangle {
				x: det.x as _,
				y: det.y as _,
				width: det.aw as _,
				height: det.ah as _,
			},
			Vector2::default(),
			degrees,
			Color::new(255, 255, 255, 255),
		);
	}
}
impl Layable for Texture {
	fn size(&self) -> (i32, i32) {
		(self.tex.width, self.tex.height)
	}
	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		self.render(d, det.mul_size(scale));
	}
}
