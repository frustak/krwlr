use anyhow::Result;
use dotenvy::dotenv;
use std::{
    collections::VecDeque,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable must be set");
    let mut que = VecDeque::from([seed]);
    let url = que.pop_front().unwrap();
    let resp = reqwest::get(url).await?.text().await?;
    let data_dir = Path::new("./data");
    if !data_dir.exists() {
        fs::create_dir(data_dir)?;
    }
    let repo_path = data_dir.join("repo.txt");
    let mut repo = if !repo_path.exists() {
        File::create(repo_path)?
    } else {
        OpenOptions::new().write(true).open(repo_path)?
    };
    repo.write_all(resp.as_bytes())?;
    Ok(())
}
