use core::str;
use std::{borrow::Cow, future::Future, str::Utf8Error};

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "fs")]
pub use fs::FsAssets;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::HttpAssets;

pub mod modifiers;
pub use modifiers::{Empty, Log};

pub use anyhow::{Error, Result};

/// the powerhouse of asset_provider
pub trait Assets {
	fn asset(&self, key: &str) -> impl Future<Output = Result<Asset>> + Send + Sync;
}

#[derive(Clone, Debug)]
pub struct Asset {
	bin: Cow<'static, [u8]>,
}
impl Asset {
	pub fn new(bin: impl Into<Cow<'static, [u8]>>) -> Self {
		let bin = bin.into();
		Self { bin }
	}

	pub fn as_str(self) -> Result<Cow<'static, str>, Utf8Error> {
		match self.bin {
			Cow::Borrowed(bytes) => {
				let s = std::str::from_utf8(bytes)?;
				Ok(Cow::Borrowed(s))
			}
			Cow::Owned(bytes) => {
				let s = String::from_utf8(bytes)
					.map_err(|e| std::str::Utf8Error::from(e.utf8_error()))?;
				Ok(Cow::Owned(s))
			}
		}
	}
	pub fn to_vec(self) -> Vec<u8> {
		match self.bin {
			Cow::Borrowed(b) => b.into(),
			Cow::Owned(a) => a,
		}
	}
}
impl AsRef<[u8]> for Asset {
	fn as_ref(&self) -> &[u8] {
		self.bin.as_ref()
	}
}
