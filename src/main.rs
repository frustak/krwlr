use anyhow::Result;
use dotenvy::dotenv;
use select::{document::Document, predicate::Name};
use std::{
    collections::{HashSet, VecDeque},
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable must be set");
    let mut que = VecDeque::from([seed.clone()]);
    let data_dir = Path::new("./data");
    if !data_dir.exists() {
        fs::create_dir(data_dir)?;
    }
    let repo_path = data_dir.join("repo.txt");
    let mut repo = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(repo_path)?;
    let mut uniq_links: HashSet<String> = HashSet::from([seed]);
    while !que.is_empty() {
        let url = que.pop_front().unwrap();
        println!("Crawling {}", url);
        let resp = reqwest::get(url).await?.text().await?;
        repo.write_all(resp.as_bytes())?;
        let links: HashSet<String> = Document::from(resp.as_str())
            .find(Name("a"))
            .filter_map(|node| node.attr("href"))
            .map(|link| link.to_string())
            .collect();
        let new_links: HashSet<String> = links.difference(&uniq_links).cloned().collect();
        uniq_links.extend(new_links.clone());
        que.extend(new_links);
    }
    Ok(())
}
