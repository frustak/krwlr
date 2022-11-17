use anyhow::Result;
use dotenvy::dotenv;
use std::{
    collections::{HashMap, VecDeque},
    env,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable must be set");
    let queue = VecDeque::from([seed]);
    let resp = reqwest::get("https://dummyjson.com/products")
        .await?
        .text()
        .await?;
    println!("{:?}", resp);
    Ok(())
}
