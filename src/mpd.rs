use anyhow::Result;
use bytes::BytesMut;
use mpd_client::{
    Client,
    client::{ConnectionEvent, Subsystem},
    commands,
    responses::PlayState,
};
use tokio::net::TcpStream;

use crate::notification;

pub async fn connect_to_mpd() -> Result<()> {
    let connection = TcpStream::connect("localhost:6600").await?;

    let (client, mut state_changes) = Client::connect(connection).await?;

    let mut handle = notification::init(SongInfo::new(&client).await?)?.show()?;

    loop {
        match state_changes.next().await {
            Some(ConnectionEvent::SubsystemChange(Subsystem::Player)) => {
                handle = notification::update(&mut handle, SongInfo::new(&client).await?)?;
            }
            Some(ConnectionEvent::SubsystemChange(_)) => (),
            _ => break,
        }
    }

    Ok(())
}

async fn get_image(client: &Client, uri: &str) -> Result<Option<BytesMut>> {
    let mut out = BytesMut::new();
    let mut expected_size = 0;
    let mut from_file = false;

    if let Some(resp) = client.command(commands::AlbumArt::new(uri)).await? {
        out = resp.data;
        expected_size = resp.size;
        out.reserve(expected_size);
        from_file = true;
    }

    if !from_file {
        if let Some(resp) = client.command(commands::AlbumArt::new(uri)).await? {
            out = resp.data;
            expected_size = resp.size;
            out.reserve(expected_size);
        } else {
            return Ok(None);
        }
    }

    while out.len() < expected_size {
        let resp = if from_file {
            client
                .command(commands::AlbumArt::new(uri).offset(out.len()))
                .await?
        } else {
            client
                .command(commands::AlbumArtEmbedded::new(uri).offset(out.len()))
                .await?
        };

        if let Some(resp) = resp {
            out.extend_from_slice(&resp.data);
        } else {
            return Ok(None);
        }
    }

    Ok(Some(out))
}

pub struct SongInfo {
    pub state: PlayState,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub album_art: Option<BytesMut>,
}

impl SongInfo {
    async fn new(client: &Client) -> Result<Self> {
        let Some(song) = client
            .command(commands::CurrentSong)
            .await?
            .map(|song| song.song)
        else {
            return Ok(SongInfo {
                state: PlayState::Stopped,
                artist: String::default(),
                album: String::default(),
                title: String::default(),
                album_art: None,
            });
        };

        Ok(SongInfo {
            state: client.command(commands::Status).await?.state,
            artist: song.artists().join(", "),
            album: song.album().unwrap_or_default().to_owned(),
            title: song.title().unwrap_or_default().to_owned(),
            album_art: get_image(client, &song.url).await?,
        })
    }

    pub fn summary(&self) -> String {
        match self.state {
            PlayState::Stopped => "Stopped",
            PlayState::Playing => "Playing:",
            PlayState::Paused => "Paused:",
        }
        .to_string()
    }

    pub fn body(&self) -> String {
        format!("<b>{}</b>\n{}\n{}", self.title, self.artist, self.album)
    }
}
