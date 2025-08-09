use std::path::PathBuf;

use anyhow::Context;
use asset_provider::Assets;
use sui::comp::text::Font;
use temp_dir::TempDir;
use tokio::io::AsyncWriteExt;

pub async fn load_font<'a, A: Assets>(
	assets: &A,
	key: &str,
	d: &mut sui::Handle<'a>,
) -> anyhow::Result<Font> {
	// let font_path = into_temp_dir(assets, key).await?;
	let asset = assets.asset(key).await?;

	let font = {
		let (rl, th) = d.to_parts_mut();

		let last_dot = key
			.bytes()
			.enumerate()
			.filter_map(|(i, c)| if c == b'.' { Some(i) } else { None })
			.last();

		let file_type = match last_dot {
			Some(a) => &key[a..],
			None => "",
		};

		let file_data = asset.as_slice();

		let font_size = 32;

		let chars = None;

		rl.load_font_from_memory(th, file_type, file_data, font_size, chars)
	}
	.with_context(|| format!("while loading font from {key}, from memory"));
	let raylib_font: sui::raylib::text::Font = font?;

	let font = sui::comp::text::Font::from_raylib(raylib_font, true);

	Ok(font)
}

pub async fn into_temp_dir<A: Assets>(assets: &A, key: &str) -> anyhow::Result<PathBuf> {
	let asset = assets.asset(key).await?;

	let dir = TempDir::new().with_context(|| format!("while loading font {key} from assets"))?;
	let font_path = dir.path().join(key);
	if let Some(parent) = font_path.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}

	{
		let mut file = tokio::fs::OpenOptions::new()
			.create(true)
			.write(true)
			.open(&font_path)
			.await
			.with_context(|| format!("while writing the font from assets to a temp directory"))?;

		file.write_all(asset.as_slice()).await?;
	}

	Ok(font_path)
}
