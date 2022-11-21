use anyhow::Result;
use dotenvy::dotenv;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use select::{document::Document, predicate::Name};
use std::{
    collections::{HashSet, VecDeque},
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};
use tracing::info;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .without_time()
        .compact()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .init();
    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable is missing!");
    let mut que = VecDeque::from([seed.clone()]);
    let max_crawl = env::var("MAX_CRAWL")
        .expect("MAX_CRAWL environment variable is missing!")
        .parse::<usize>()
        .expect("MAX_CRAWL environment variable is not a number!");
    let mut crawled_count = 0;
    let mut uniq_links: HashSet<String> = HashSet::from([seed]);
    let mut repo = open_repo()?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    info!("Starting the crawl");
    while !que.is_empty() && crawled_count < max_crawl {
        let url = que.pop_front().unwrap();
        info!(url);
        let resp = reqwest::get(url).await?.text().await?;
        encoder.write_all(resp.as_bytes())?;
        let new_links: HashSet<String> = Document::from(resp.as_str())
            .find(Name("a"))
            .filter_map(|node| node.attr("href"))
            .filter(|link| Url::parse(link).is_ok())
            .map(|link| link.to_string())
            .collect::<HashSet<String>>()
            .difference(&uniq_links)
            .cloned()
            .collect();
        uniq_links.extend(new_links.clone());
        new_links.into_iter().for_each(|link| que.push_back(link));
        crawled_count += 1;
    }
    let compressed = encoder.finish()?;
    repo.write_all(&compressed)?;
    info!("Crawl complete");
    Ok(())
}

fn open_repo() -> Result<File> {
    let data_dir = Path::new("./data");
    fs::create_dir_all(data_dir)?;
    let repo_path = data_dir.join("repo.zlib");
    let repo = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(repo_path)?;
    Ok(repo)
}
