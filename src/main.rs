use anyhow::{anyhow, bail, Result};
use dotenvy::dotenv;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use reqwest::Response;
use scraper::{Html, Selector};
use std::{
    collections::{HashSet, VecDeque},
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
use tracing::info;
use url::Url;

fn setup_logger() {
    tracing_subscriber::fmt()
        .pretty()
        .without_time()
        .compact()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .init();
}

fn get_env_vars() -> (String, usize) {
    dotenv().unwrap();
    let seed = env::var("SEED").expect("SEED environment variable is missing!");
    let max_crawl = env::var("MAX_CRAWL")
        .expect("MAX_CRAWL environment variable is missing!")
        .parse::<usize>()
        .expect("MAX_CRAWL environment variable is not a number!");
    (seed, max_crawl)
}

fn is_html(response: &Response) -> bool {
    response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|content_type| content_type.contains("text/html"))
        .unwrap_or(false)
}

fn is_same_host(url: &Url, host: &str) -> bool {
    url.host_str()
        .map(|url_host| url_host == host)
        .unwrap_or(false)
}

fn parse_urls(document: &str) -> Vec<Url> {
    let anchor_selector = Selector::parse("a").unwrap();
    Html::parse_document(document)
        .select(&anchor_selector)
        .filter_map(|node| node.value().attr("href"))
        .filter_map(|url| Url::parse(url).ok())
        .collect()
}

fn get_host(url_str: &str) -> String {
    Url::parse(url_str)
        .expect("URL is invalid!")
        .host_str()
        .expect("URL is missing a host!")
        .to_string()
}

#[tokio::main]
async fn main() {
    setup_logger();
    let (seed, max_crawl) = get_env_vars();
    info!("seed: {}, max crawl: {}", seed, max_crawl);
    let mut crawler = Crawler::new(&seed, max_crawl);
    info!("Starting the crawl");
    crawler.ignite().await;
    info!("Crawl complete");
    info!("{:#?}", crawler.metrics);
}

#[derive(Debug)]
struct Metrics {
    total_urls: usize,
    other_hosts: usize,
    same_hosts: usize,
    total_html_files: usize,
    downloaded_bytes: usize,
    compressed_bytes: usize,
    uncompressed_bytes: usize,
    fetch_count: usize,
    process_time: f64,
    download_time: f64,
}

impl Metrics {
    fn new() -> Self {
        Self {
            total_urls: 1,
            total_html_files: 0,
            same_hosts: 1,
            downloaded_bytes: 0,
            compressed_bytes: 0,
            uncompressed_bytes: 0,
            other_hosts: 0,
            fetch_count: 0,
            process_time: 0.0,
            download_time: 0.0,
        }
    }
}

struct Crawler {
    repo: Repository,
    que: VecDeque<String>,
    uniq_urls: HashSet<String>,
    crawled_count: usize,
    max_crawl: usize,
    request: reqwest::Client,
    seed_host: String,
    metrics: Metrics,
}

impl Crawler {
    fn new(seed: &str, max_crawl: usize) -> Self {
        Self {
            repo: Repository::open().unwrap(),
            que: VecDeque::from([seed.to_string()]),
            uniq_urls: HashSet::from([seed.to_string()]),
            crawled_count: 0,
            max_crawl,
            request: reqwest::Client::new(),
            seed_host: get_host(seed),
            metrics: Metrics::new(),
        }
    }

    fn should_crawl(&self) -> bool {
        !self.que.is_empty() && self.crawled_count < self.max_crawl
    }

    fn next_url(&mut self) -> Option<String> {
        self.que.pop_front()
    }

    fn add_new_urls(&mut self, urls: HashSet<String>) {
        urls.iter().cloned().for_each(|url| self.que.push_back(url));
    }

    async fn crawl(&mut self) -> Result<()> {
        let url = self
            .next_url()
            .ok_or_else(|| anyhow!("Crawler queue is empty!"))?;
        info!(url);
        let start = Instant::now();
        let resp = self.request.get(url).send().await?;
        self.metrics.download_time += start.elapsed().as_secs_f64();
        self.metrics.fetch_count += 1;
        self.metrics.downloaded_bytes += resp.content_length().unwrap_or(0) as usize;
        if !is_html(&resp) {
            bail!("Response is not HTML");
        }
        self.metrics.total_html_files += 1;
        let body = resp.text().await?;
        self.repo.store(&body);
        let all_urls = parse_urls(&body);
        let all_urls_count = all_urls.len();
        self.metrics.total_urls += all_urls_count;
        let same_host_urls: HashSet<String> = all_urls
            .into_iter()
            .filter(|url| is_same_host(url, &self.seed_host))
            .map(|url| url.to_string())
            .collect();
        self.metrics.other_hosts += all_urls_count - same_host_urls.len();
        self.metrics.same_hosts += same_host_urls.len();
        let new_urls: HashSet<String> = same_host_urls
            .difference(&self.uniq_urls)
            .cloned()
            .collect();
        self.uniq_urls.extend(new_urls.clone());
        self.add_new_urls(new_urls);
        self.crawled_count += 1;
        Ok(())
    }

    async fn ignite(&mut self) {
        let start = Instant::now();
        while self.should_crawl() {
            self.crawl().await.ok();
        }
        self.metrics.compressed_bytes = self.repo.compressed.metadata().unwrap().len() as usize;
        self.metrics.uncompressed_bytes = self.repo.uncompressed.metadata().unwrap().len() as usize;
        self.metrics.process_time = start.elapsed().as_secs_f64();
    }
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

    fn store(&mut self, content: &str) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(content.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        self.compressed.write_all(&compressed).unwrap();
        self.uncompressed.write_all(content.as_bytes()).unwrap();
    }
}
