# Krwlr - A Crawler Written In Rust

## What it does
This is a simple crawler, it starts to crawl at the given seed and only crawls within the seed domain.
The downloaded webpages are stored in `./data/` directory.


## Requirements
- Rust
- Cargo

## Setup
Create a `.env` file and set the required environments variables. an example is found in the `.env-example` file.

## How to run
```bash
cargo run --release
```
