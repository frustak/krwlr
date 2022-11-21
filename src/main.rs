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
    let mut repo = open_repo()?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    let anchor_selector = Selector::parse("a").unwrap();
    let client = reqwest::Client::new();
    info!("Starting the crawl");
    while !que.is_empty() && crawled_count < max_crawl {
        let url = que.pop_front().unwrap();
        info!(url);
        let resp = client.get(url).send().await.ok();
        if let Some(resp) = resp {
            let resp = resp.text().await.ok();
            if let Some(resp) = resp {
                encoder.write_all(resp.as_bytes())?;
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
    let compressed = encoder.finish()?;
    repo.write_all(&compressed)?;
    info!("Crawl complete");
    Ok(())
}

fn open_repo() -> Result<File> {
    let data_dir = Path::new("./data");
    fs::create_dir_all(data_dir)?;
    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let file_name = format!("repo-{}.zlib", now);
    let repo_path = data_dir.join(file_name);
    let repo = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(repo_path)?;
    Ok(repo)
}
