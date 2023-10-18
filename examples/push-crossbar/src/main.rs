use unit_crossbar_client::Crossbar;
use unit_utils::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut crossbar =
        Crossbar::new("http://127.0.0.1:6448".to_owned(), "ilovecats".to_owned()).await?;

    loop {
        crossbar
            .push_text("test".to_owned(), "I love cats".to_owned())
            .await?;

        println!("sent message");

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
