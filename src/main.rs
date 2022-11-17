use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable must be set");
    println!("{}", seed);
}
