use reqwest;
use tokio::{macros, spawn};
use futures::executor::block_on;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// A space-delimited list of substack sites to scrape, such as "https://blog.bytebytego.com/ https://astralcodexten.substack.com/"
    #[clap(short, long, use_value_delimiter = true, value_delimiter = ' ')]
    websites: Vec<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli.websites);

    println!("---Beginning scraper---");
    let join_handle = tokio::spawn(async move {
        // Process each socket concurrently.
        scrape().await;
    });
    // Wait for the async functions to complete.
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
