mod mpd;
mod notification;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = mpd::connect_to_mpd().await;
}
