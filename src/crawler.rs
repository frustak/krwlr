use crate::{metrics::Metrics, repository::Repository};
use anyhow::{anyhow, bail, Ok, Result};
use reqwest::Response;
use scraper::{Html, Selector};
use std::{
    collections::{HashSet, VecDeque},
    time::Instant,
};
use tracing::info;
use url::Url;

// TODO: Use builder pattern to create a crawler
pub struct Crawler {
    repo: Repository,
    que: VecDeque<String>,
    uniq_urls: HashSet<String>, // TODO: change to checked urls
    crawled_count: usize,       // TODO: remove this, should be calculated from checked urls
    max_crawl: usize,
    request: reqwest::Client,
    seed_domain: String,
    metrics: Metrics,
}

impl Crawler {
    pub fn new(seed: &str, max_crawl: usize) -> Self {
        Self {
            repo: Repository::new(),
            que: VecDeque::from([seed.to_string()]),
            uniq_urls: HashSet::from([seed.to_string()]),
            crawled_count: 0,
            max_crawl,
            request: reqwest::Client::new(),
            seed_domain: get_domain(seed),
            metrics: Metrics::new(),
        }
    }

    pub async fn ignite(&mut self) {
        let start = Instant::now();
        while self.should_crawl() {
            self.crawl().await.ok();
        }
        self.metrics.compressed_bytes = self.repo.compressed.metadata().unwrap().len() as usize;
        self.metrics.uncompressed_bytes = self.repo.uncompressed.metadata().unwrap().len() as usize;
        self.metrics.process_time = start.elapsed().as_secs_f64();
        self.metrics.que_size_at_end = self.que.len();
    }

    pub fn show_metrics(&self) {
        info!("{:#?}", self.metrics);
    }

    fn should_crawl(&self) -> bool {
        !self.que.is_empty() && self.crawled_count < self.max_crawl
    }

    // TODO: refactor to chunks
    async fn crawl(&mut self) -> Result<()> {
        let url = self
            .next_url()
            .ok_or_else(|| anyhow!("Crawler queue is empty!"))?;
        info!(url);
        let start = Instant::now();
        let resp = self.request.get(&url).send().await?;
        self.metrics.download_time += start.elapsed().as_secs_f64();
        self.metrics.fetch_count += 1;
        self.metrics.downloaded_bytes += resp.content_length().unwrap_or(0) as usize;
        if !is_html(&resp) {
            bail!("Response is not HTML");
        }
        self.metrics.total_html_files += 1;
        let body = resp.text().await?;
        self.repo.store(&body);
        let all_urls = parse_urls(&body, &url);
        let all_urls_count = all_urls.len();
        self.metrics.total_urls += all_urls_count;
        let same_domain_urls: HashSet<String> = all_urls
            .into_iter()
            .filter(|url| is_same_domain(url, &self.seed_domain))
            .map(|url| url.to_string())
            .collect();
        self.metrics.other_domains += all_urls_count - same_domain_urls.len();
        self.metrics.same_domains += same_domain_urls.len();
        let new_urls: HashSet<String> = same_domain_urls
            .difference(&self.uniq_urls)
            .cloned()
            .collect();
        self.uniq_urls.extend(new_urls.clone());
        self.add_new_urls(new_urls);
        self.crawled_count += 1;
        Ok(())
    }

    fn next_url(&mut self) -> Option<String> {
        self.que.pop_front()
    }

    fn add_new_urls(&mut self, urls: HashSet<String>) {
        urls.iter().cloned().for_each(|url| self.que.push_back(url));
    }
}

fn get_domain(url_str: &str) -> String {
    Url::parse(url_str)
        .expect("URL is invalid!")
        .domain()
        .expect("URL is missing domain!")
        .to_string()
}

fn is_html(response: &Response) -> bool {
    response
        .headers()
        .get("content-type")
        .and_then(|content_type| content_type.to_str().ok())
        .map(|content_type| content_type.contains("text/html"))
        .unwrap_or(false)
}

fn parse_urls(document: &str, base_url: &str) -> Vec<Url> {
    let anchor_selector = Selector::parse("a").unwrap();
    Html::parse_document(document)
        .select(&anchor_selector)
        .filter_map(|node| node.value().attr("href"))
        .filter_map(|url| get_absolute_url(url, base_url))
        .collect()
}

fn get_absolute_url(url: &str, base_url: &str) -> Option<Url> {
    match Url::parse(url) {
        std::result::Result::Ok(url) => Some(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => to_absolute_url(url, base_url).ok(),
        _ => None,
    }
}

fn to_absolute_url(url: &str, base: &str) -> Result<Url> {
    let base_url = Url::parse(base)?;
    let absolute_url = base_url.join(url)?;
    Ok(absolute_url)
}

fn is_same_domain(url: &Url, domain: &str) -> bool {
    url.domain()
        .map(|url_domain| url_domain == domain)
        .unwrap_or(false)
}
