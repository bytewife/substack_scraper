#![feature(file_create_new)]

use reqwest;
use html2text::from_read;
use serde::{Deserialize, Serialize};
use voca_rs::strip::strip_tags;
use tokio::{macros, spawn};
use futures::executor::block_on;
use log::{debug, LevelFilter};
use env_logger::{Builder, Target};
use std::{env, fs, iter};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use chrono::Local;
use scraper::{Html, Selector};

use clap::{Parser, Subcommand};
use clap::builder::TypedValueParser;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use env_logger::Target::Stdout;
use futures::TryFutureExt;
use fancy_regex::Regex;
use reqwest::Url;

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
    //Convert to Url type
    let websites = cli.websites.into_iter().map(|s|
        Url::parse(&s)
            .unwrap())
            .collect::<Vec<Url>>();

    let join_handle = tokio::spawn(async move {
        for website in websites {
            scrape(&website).await.expect(fmt!("Failed to scrape {}", website));
        }
    });

    // Wait for the async functions to complete.
    join_handle.await.unwrap();
    Ok(())
}

#[derive(Deserialize)]
#[derive(Debug)]
struct CanonicalUrl {
    canonical_url: Url,
}

async fn scrape(homepage_url: &Url) -> eyre::Result<()> {
    let post_urls = get_post_urls(homepage_url).await?;

    let mut urls_to_post_content: Vec<(&Url, Vec<String>)> = Vec::new();

    // Get posts' content.
    for mut post_url in &post_urls {
        let post = get_post_content(&post_url).await?;
        urls_to_post_content.push((&post_url, post));
    }

    let blog_folder_path = Path::new("blogs").join(Path::new(&homepage_url.host_str().unwrap()));

    // Write to files.
    for (url, post) in urls_to_post_content {
        let path = Path::new(url.path());
        let path = path.strip_prefix("/").unwrap_or(path);
        let path = blog_folder_path.join(path);
        if let Some(dir) = path.parent() {
            fs_err::create_dir_all(dir)?;
        }
        fs_err::write(&path, post.join("\n").as_bytes())?;
    }
    Ok(())
}

/// Get the text content of a post.
async fn get_post_content(url: &Url) -> eyre::Result<Vec<String>> {
    // TODO wait & retry getting content when hitting rate limit.
    println!("url is {:?}", url);

    let mut result = Vec::new();
    loop {
        let headers = reqwest::get(url.clone()).await?;
        println!("headers are {:?}", headers);
        let mut body = headers.text().await?;

        let fragment = Html::parse_fragment(&body);
        // The following selector looks for <p> elements with the .available-content parent.
        let selector = Selector::parse(".available-content p:not(.button-wrapper)").unwrap();
        for it in fragment.select(&selector) {
            let temp = it.inner_html();
            result.push(cleanup_content(&temp));
        };
        if !result.is_empty() { break };
        // Wait on rate limiter.
        sleep(std::time::Duration::from_secs(1));
        println!("Retrying...");
    }
    println!("{:?}", result);
    Ok(result)
}

/// Transform HTML into clean text output.
fn cleanup_content(input: &String) -> String {
    // Replace in-paragraph footnote links with "". Assumes that the following regex works.
    let regex_footnote = Regex::new(r">\d</a>").unwrap();
    let temp = regex_footnote.replace_all(&input, "></a>").to_string();
    // Strip HTML tags.
    let temp = strip_tags(&temp);
    // Remove HTML encoding artifacts like ;nbsp;
    let temp = from_read(temp.as_bytes(), 100);
    let temp = temp.replace("\n", " ");
    temp
}

async fn get_post_urls(homepage_url: &Url) -> eyre::Result<HashSet<Url>> {
    debug!("Scraping {}", homepage_url);

    // Current page number.
    let mut page_offset = 0;
    // Pages to request on each iteration.
    let page_limit = 12;

    // Contains the hashset of article URLs.
    let mut seen_urls = HashSet::new();

    loop {
        // Get content. The api url may be subject to change from Substack.
        let current_request_url = format!("{}api/v1/archive?sort=new&search=&offset={}&limit={}", homepage_url, page_offset, page_limit);
        debug!("current_request_url = {}", &current_request_url);

        let post_urls = reqwest::get(&current_request_url)
            .await?.
            json::<Vec<CanonicalUrl>>()
            .await?;

        // Add page URLs.
        // Exit on empty query.
        if post_urls.is_empty() {
            break;
        }
        seen_urls.extend(post_urls.into_iter().map(|it| it.canonical_url));

        page_offset += page_limit;
    }
    debug!("seen_urls = {seen_urls:?}");

    debug!("Finished scraping {}", homepage_url);
    Ok(seen_urls)
}
