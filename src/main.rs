mod cache;
mod mpd;
mod notification;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    mpd::connect_to_mpd().await.unwrap();
}
