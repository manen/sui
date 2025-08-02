pub use image;

use asset_provider::Assets;
use image::DynamicImage;
use sui::{raylib::texture::Image, tex::Texture};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	Asset(#[from] asset_provider::Error),
	#[error("image error while loading an asset:\n{0}")]
	Image(#[from] image::ImageError),
	#[error("error in LoadTextureFromImage:\n{0}")]
	LoadTextureFromImage(#[from] sui::raylib::error::LoadTextureError),

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
	fn texture(&self, d: &mut sui::Handle) -> Result<Texture>;
}
impl ImageExt for DynamicImage {
	fn texture(&self, d: &mut sui::Handle) -> Result<Texture> {
		let rgba = self.to_rgba8();
		let (w, h) = rgba.dimensions();

		let pixels = rgba.into_raw();

		let image = unsafe {
			Image::from_raw(sui::raylib::ffi::Image {
				data: pixels.as_ptr() as *mut std::ffi::c_void,
				width: w as _,
				height: h as _,
				mipmaps: 1,
				format: sui::raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as _,
			})
		};
		std::mem::forget(pixels); // <- pixels is managed by image now

		let (d, thread) = d.to_parts_mut();

		let texture = d.load_texture_from_image(thread, &image)?;
		let texture = Texture::new_from_raylib(texture);

		Ok(texture)
	}
}
