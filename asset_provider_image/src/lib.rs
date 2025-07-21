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

	#[error("JoinError when trying to load the image from memory using spawn_blocking")]
	JoinError(#[from] tokio::task::JoinError),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait AssetsExt {
	fn asset_image(&self, key: &str) -> impl Future<Output = Result<DynamicImage>> + Send + Sync;
}
impl<A: Assets + Sync> AssetsExt for A {
	async fn asset_image(&self, key: &str) -> Result<DynamicImage> {
		let asset = self.asset(key).await?;
		let asset = asset.to_vec();
		let handle = tokio::runtime::Handle::current();
		let image = handle
			.spawn_blocking(move || image::load_from_memory(&asset))
			.await??;

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

		// na figyu mar compileolunk bar a game loadingnal nem lett meg rendesen az async szoval paraszt modjara
		// blockolva toltjuk be a texturakat
		// de ez a copy_nonoverlapping errorol most es mar basz fel rendesen
		// unsafe {
		// 	std::ptr::copy_nonoverlapping(
		// 		rgba.as_ptr() as *const u8,
		// 		image.data as *mut u8,
		// 		rgba.len(),
		// 	);
		// }

		let texture = d
			.load_texture_from_image(thread, &image)
			.map_err(Error::LoadTextureFromImage)?;
		let texture = Texture::new_from_raylib(texture);

		Ok(texture)
	}
}
