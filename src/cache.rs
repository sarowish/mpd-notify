use anyhow::{Result, bail};
use bytes::BytesMut;
use std::path::PathBuf;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_cache_dir() -> Result<PathBuf> {
    let path = match dirs::cache_dir() {
        Some(path) => path.join(PACKAGE_NAME),
        None => bail!("Couldn't find cache directory"),
    };

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

fn hash_image(bytes: &BytesMut) -> String {
    let mut s = blake3::Hasher::new();
    s.update(bytes);
    s.update(b"v1");

    let mut out = [0u8; 12];
    s.finalize_xof().fill(&mut out);

    hex::encode(out)
}

pub fn get_cached_image_path(bytes: &BytesMut) -> Result<PathBuf> {
    let hash = hash_image(bytes);

    let mut path = get_cache_dir()?.join(hash);
    path.set_extension("jpg");

    Ok(path)
}
