use std::collections::HashSet;
use reqwest;

use serde::{Deserialize, Serialize};
use tokio::{macros, spawn};
use futures::executor::block_on;
use log::{debug, LevelFilter};
use env_logger::{Builder, Target};
use std::{env, iter};
use std::io::Write;
use chrono::Local;
use scraper::{Html, Selector};

use clap::{Parser, Subcommand};
use clap::builder::TypedValueParser;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use env_logger::Target::Stdout;
use futures::TryFutureExt;

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
async fn main() -> eyre::Result<()> {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .target(Stdout)
        .init();

    let cli = Cli::parse();

    debug!("Websites are {:?}", cli.websites);

    for website in cli.websites {
        // TODO flip these loops lmao
    //     let join_handle = tokio::spawn(async move {
    //         // Process each socket concurrently.
            scrape(website).await?;
        // });

        // Wait for the async functions to complete.
        // join_handle.await.unwrap()
    }
    Ok(())
}

#[derive(Deserialize)]
#[derive(Debug)]
struct CanonicalUrl {
    canonical_url: String,
}

async fn scrape(homepage_url: String) -> eyre::Result<()> {
    let urls = get_blog_urls(homepage_url).await?;
    debug!("Urls are {:?}", urls);
    // Get posts' content.
    Ok(())
}

async fn get_blog_urls(homepage_url: String) -> eyre::Result<HashSet<String>> {
    debug!("Scraping {}", homepage_url);

    // Current page number.
    let mut page_offset = 0;
    // Pages to request on each iteration.
    let page_limit = 12;

    // Contains the hashset of article URLs.
    let mut seen_urls = HashSet::new();

    loop {
        // Get content.
        let current_request_url = format!("{}api/v1/archive?sort=new&search=&offset={}&limit={}", homepage_url, page_offset, page_limit);
        debug!("current_request_url = {}", &current_request_url);

        let page_urls = reqwest::get(&current_request_url)
            .await?.
            json::<Vec<CanonicalUrl>>()
            .await?;
        debug!("body = {:?}", &page_urls);

        // Add page URLs.
        // Exit on empty query.
        if page_urls.is_empty() {
            break;
        }
        seen_urls.extend(page_urls.into_iter().map(|it| it.canonical_url));

        page_offset += page_limit;
    }
    debug!("seen_urls = {seen_urls:?}");

    debug!("Finished scraping {}", homepage_url);
    Ok(seen_urls)
}
