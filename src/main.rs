use anyhow::Result;
use dotenvy::dotenv;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use scraper::{Html, Selector};
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
    let host_url = Url::parse(&seed).expect("SEED is not a valid URL!");
    let seed_host = host_url.host_str().expect("SEED is missing a host!");
    let max_crawl = env::var("MAX_CRAWL")
        .expect("MAX_CRAWL environment variable is missing!")
        .parse::<usize>()
        .expect("MAX_CRAWL environment variable is not a number!");
    let mut que = VecDeque::from([seed.clone()]);
    let mut uniq_links: HashSet<String> = HashSet::from([seed]);
    let mut crawled_count = 0;
    let mut repo = Repository::open()?;
    let anchor_selector = Selector::parse("a").unwrap();
    let client = reqwest::Client::new();
    info!("Starting the crawl");
    while !que.is_empty() && crawled_count < max_crawl {
        let url = que.pop_front().unwrap();
        info!(url);
        let resp = client.get(url).send().await.ok();
        if let Some(resp) = resp {
            let is_html = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|content_type| content_type.contains("text/html"))
                .unwrap_or(false);
            if !is_html {
                continue;
            }
            let resp = resp.text().await.ok();
            if let Some(resp) = resp {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(resp.as_bytes())?;
                let compressed = encoder.finish()?;
                repo.compressed.write_all(&compressed)?;
                repo.uncompressed.write_all(resp.as_bytes())?;
                let new_links: HashSet<String> = Html::parse_document(resp.as_str())
                    .select(&anchor_selector)
                    .filter_map(|node| node.value().attr("href"))
                    .filter_map(|link| Url::parse(link).ok())
                    .filter(|link| {
                        link.host_str()
                            .map(|host| seed_host == host)
                            .unwrap_or(false)
                    })
                    .map(|link| link.to_string())
                    .collect::<HashSet<String>>()
                    .difference(&uniq_links)
                    .cloned()
                    .collect();
                uniq_links.extend(new_links.clone());
                new_links.into_iter().for_each(|link| que.push_back(link));
                crawled_count += 1;
            }
        }
    }
    info!("Crawl complete");
    Ok(())
}

struct Repository {
    compressed: File,
    uncompressed: File,
}

impl Repository {
    fn open() -> Result<Self> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;
        let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let compressed_name = format!("repo-{}.zlib", now);
        let compressed_path = data_dir.join(compressed_name);
        let compressed = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(compressed_path)?;
        let uncompressed_name = format!("repo-{}.txt", now);
        let uncompressed_path = data_dir.join(uncompressed_name);
        let uncompressed = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(uncompressed_path)?;
        Ok(Self {
            compressed,
            uncompressed,
        })
    }
}
