use crate::{metrics::Metrics, repository::Repository, timer::Timer};
use anyhow::{bail, Context, Ok, Result};
use rayon::prelude::*;
use reqwest::Response;
use scraper::{Html, Selector};
use std::{
    collections::{HashSet, VecDeque},
    time::Instant,
};
use tracing::info;
use url::Url;

pub struct Crawler {
    repo: Repository,
    que: VecDeque<String>,
    uniq_urls: HashSet<String>,
    crawled_count: usize,
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
            let crawl_result = self.crawl().await;
            if let Err(error) = crawl_result {
                let error_string = error.to_string();
                let error_count = self.metrics.error.errors.entry(error_string).or_insert(0);
                *error_count += 1;
            }
        }
        self.metrics.log.compressed_bytes = self.repo.compressed.metadata().unwrap().len() as usize;
        self.metrics.log.uncompressed_bytes =
            self.repo.uncompressed.metadata().unwrap().len() as usize;
        self.metrics.log.process_time = start.elapsed().as_secs_f64();
        self.metrics.log.que_size_at_end = self.que.len();
    }

    pub fn show_metrics(&self) {
        info!("{:#?}", self.metrics);
    }

    fn should_crawl(&self) -> bool {
        !self.que.is_empty() && self.crawled_count < self.max_crawl
    }

    async fn crawl(&mut self) -> Result<()> {
        let url = self.next_url()?;
        info!(url);
        let body = self.fetch_next(&url).await?;
        self.repo.store(&body);
        let all_urls = parse_urls(&body, &url);
        let all_urls_count = all_urls.len();
        self.metrics.log.total_urls += all_urls_count;
        let same_domain_urls = self.same_domain_urls(all_urls);
        self.metrics.log.other_domains += all_urls_count - same_domain_urls.len();
        self.metrics.log.same_domains += same_domain_urls.len();
        let new_urls = self.new_urls(same_domain_urls);
        self.uniq_urls.extend(new_urls.clone());
        self.push_new_urls(new_urls);
        self.crawled_count += 1;
        Ok(())
    }

    fn new_urls(&mut self, urls: HashSet<String>) -> HashSet<String> {
        let new_urls: HashSet<String> = urls.difference(&self.uniq_urls).cloned().collect();
        new_urls
    }

    fn same_domain_urls(&mut self, all_urls: Vec<Url>) -> HashSet<String> {
        let same_domain_urls: HashSet<String> = all_urls
            .into_par_iter()
            .filter(|url| is_same_domain(url, &self.seed_domain))
            .map(|url| url.to_string())
            .collect();
        same_domain_urls
    }

    async fn fetch_next(&mut self, url: &str) -> Result<String> {
        let timer = Timer::new();
        let resp = self.request.get(url).send().await?;
        self.metrics.log.download_time += timer.elapsed();
        self.metrics.log.fetch_count += 1;
        if !is_html(&resp) {
            bail!("Response is not HTML");
        }
        self.metrics.log.total_html_files += 1;
        let body = resp.text().await?;
        Ok(body)
    }

    fn next_url(&mut self) -> Result<String> {
        self.que.pop_front().context("No more urls to crawl")
    }

    fn push_new_urls(&mut self, urls: HashSet<String>) {
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
