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

pub struct SongInfo {
    pub state: PlayState,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub album_art: Option<(BytesMut, Option<String>)>,
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
            album_art: client.album_art(&song.url).await?,
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
