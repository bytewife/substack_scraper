#![feature(file_create_new)]

use reqwest;

use serde::{Deserialize, Serialize};
use voca_rs::strip::strip_tags;
use tokio::{macros, spawn};
use futures::executor::block_on;
use log::{debug, LevelFilter};
use env_logger::{Builder, Target};
use std::{env, iter};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use scraper::{Html, Selector};

use clap::{Parser, Subcommand};
use clap::builder::TypedValueParser;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use env_logger::Target::Stdout;
use futures::TryFutureExt;
use fancy_regex::Regex;

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
            scrape(&website).await?;
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

async fn scrape(homepage_url: &String) -> eyre::Result<()> {
    let post_urls = get_post_urls(homepage_url).await?;

    // Result.
    let mut url_to_posts: Vec<(&String, Vec<String>)> = Vec::new();

    // Get posts' content.
    for mut post_url in &post_urls {
        let post = get_post_content(&post_url).await?;
        url_to_posts.push((&post_url, post));
        // let post = get_post_content("https://etiennefd.substack.com/p/how-to-slow-down-progress-according".to_string()).await?;
    }

    let blog_folder_name = Path::new("blogs").join(&homepage_url.replace("https://", "").replace("/", "-"));

    // Check that the url filenames do not exist.
    for (url, _) in &url_to_posts {
        let filename = url_to_filename(url);
        if std::path::Path::new(&blog_folder_name).join(&filename).exists() {
            return Err(eyre!("File {} already exists. Please delete it before running this program again.", filename));
        }
    }

    // TODO fix this.
    // Write to files, overwriting if exists.
    // Create folder if it doesn't exist.
        std::fs::create_dir_all(&blog_folder_name)?;
    for (url, post) in url_to_posts {
        let filename = url_to_filename(&url);
        let path = std::path::Path::new(&blog_folder_name).join(filename);
        let mut file = std::fs::File::create_new(path)?;
        for line in post {
            file.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

fn url_to_filename(url: &String) -> String {
    return url.to_owned() + ".txt";
}

/// Get the text content of a post.
async fn get_post_content(url: &String) -> eyre::Result<Vec<String>> {
    // TODO wait & retry getting content when hitting rate limit.
    println!("url is {:?}", url);
    let body = reqwest::get(url).await?.text().await?;
    let fragment = Html::parse_fragment(&body);
    // The following selector looks for <p> elements with the .available-content parent.
    let selector = Selector::parse(".available-content p").unwrap();
    let mut result = Vec::new();
    for it in fragment.select(&selector) {
        let temp = it.inner_html();
        result.push(cleanup_content(&temp));
    };
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

    // Replace &nbsp; with space.
    temp.replace("&nbsp;", " ")
}

async fn get_post_urls(homepage_url: &String) -> eyre::Result<HashSet<String>> {
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
