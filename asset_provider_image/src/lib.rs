use std::io::Read;

pub use image;

use asset_provider::Assets;
use image::DynamicImage;
use sui::{
	raylib::{RaylibThread, ffi::PixelFormat, texture::Image},
	tex::Texture,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	Asset(#[from] asset_provider::Error),
	#[error("image error while loading an asset:\n{0}")]
	Image(#[from] image::ImageError),
	#[error("error in LoadTextureFromImage:\n{0}")]
	LoadTextureFromImage(String),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait AssetsExt {
	fn asset_image(&self, key: &str) -> impl Future<Output = Result<DynamicImage>> + Send + Sync;
}
impl<A: Assets + Sync> AssetsExt for A {
	async fn asset_image(&self, key: &str) -> Result<DynamicImage> {
		let asset = self.asset(key).await?;
		let image = image::load_from_memory(asset.as_ref())?;

		Ok(image)
	}
}

pub trait ImageExt {
	fn texture(&self, d: &mut sui::Handle, thread: &RaylibThread) -> Result<Texture>;
}
impl ImageExt for DynamicImage {
	fn texture(&self, d: &mut sui::Handle, thread: &RaylibThread) -> Result<Texture> {
		let rgba = self.to_rgba8();

		let mut image = Image::gen_image_color(
			rgba.width() as _,
			rgba.height() as _,
			sui::color(0, 0, 0, 0),
		);
		image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
		unsafe {
			std::ptr::copy_nonoverlapping(
				rgba.as_ptr(),
				image.data as *mut u8,
				rgba.bytes().count(),
			);
		}

		let texture = d
			.load_texture_from_image(thread, &image)
			.map_err(Error::LoadTextureFromImage)?;
		let texture = Texture::new_from_raylib(texture);

		Ok(texture)
	}
}
