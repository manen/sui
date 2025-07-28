use anyhow::anyhow;

use crate::{Asset, Assets, Result};
use std::{io::ErrorKind, path::PathBuf};

#[derive(Clone, Debug)]
/// implementor of [crate::Assets] that reads assets straight from the filesystem
pub struct FsAssets {
	dir: PathBuf,
}
impl FsAssets {
	pub fn new(dir: impl Into<PathBuf>) -> Result<Self> {
		let dir: PathBuf = dir.into();

		if dir.exists() {
			Ok(Self { dir })
		} else {
			Err(anyhow!(
				"attempted to create an FsAssets in a directory that doesn't exist: {}",
				dir.display()
			))
		}
	}
}
impl Assets for FsAssets {
	async fn asset(&self, key: &str) -> Result<crate::Asset> {
		let path = self.dir.join(key);
		let vec = tokio::fs::read(&path).await?;

		Ok(Asset::new(vec))
	}
}
