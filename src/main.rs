use reqwest;
use tokio::{macros, spawn};
use futures::executor::block_on;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("---Beginning scraper---");
    let join_handle = tokio::spawn(async move {
        // Process each socket concurrently.
        scrape().await;
    });
    // Wait for the async tas to complete.
    join_handle.await.unwrap()
}

async fn scrape() -> Result<(), reqwest::Error> {
let body = (match reqwest::get("https://www.google.com/")
        .await {
            Ok(res) => res.text().await?,
            Err(e) => {
                e.to_string()
            },
        });

    println!("body = {:?}", body);
    println!("---End of scraper---");
    Ok(())
}
