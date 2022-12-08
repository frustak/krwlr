use dotenvy::dotenv;
use krwlr::Crawler;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logger();
    let (seed, max_crawl) = get_env_vars();
    info!("seed: {}, max crawl: {}", seed, max_crawl);
    let mut crawler = Crawler::new(&seed, max_crawl);
    info!("Starting the crawl");
    crawler.start().await;
    info!("Crawl complete");
    crawler.show_metrics();
}

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
    dotenv().expect("Failed to load .env file");
    let seed = env::var("SEED").expect("SEED environment variable is missing!");
    let max_crawl = env::var("MAX_CRAWL")
        .expect("MAX_CRAWL environment variable is missing!")
        .parse::<usize>()
        .expect("MAX_CRAWL environment variable is not a number!");
    (seed, max_crawl)
}
