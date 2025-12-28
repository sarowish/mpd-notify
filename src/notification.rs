use crate::mpd::SongInfo;
use anyhow::Result;
use bytes::BytesMut;
use image::{GenericImageView, imageops::FilterType};
use mpd_client::responses::PlayState;
use notify_rust::{Image, Notification, NotificationHandle};

pub fn init(song: SongInfo) -> Result<Notification> {
    let mut n = Notification::new()
        .summary(&song.summary())
        .body(&match song.state {
            PlayState::Stopped => String::new(),
            _ => song.body(),
        })
        .timeout(5000)
        .finalize();

    if let Some((art, _)) = song.album_art {
        n.image_data(to_image(&art)?);
    }

    Ok(n)
}

pub fn update(handle: &mut NotificationHandle, song: SongInfo) -> Result<NotificationHandle> {
    Ok(init(song)?.id(handle.id()).show()?)
}

fn to_image(bytes: &BytesMut) -> Result<Image> {
    let mut image = image::load_from_memory(bytes)?;
    image = image.resize(128, 128, FilterType::Gaussian);

    let (width, height) = image.dimensions();

    Ok(Image::from_rgb(
        width.cast_signed(),
        height.cast_signed(),
        image.into_bytes(),
    )?)
}
