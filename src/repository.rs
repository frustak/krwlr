use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

const REPOSITORY_DIR: &str = "./data";

pub struct Repository {
    pub compressed: File,
    pub uncompressed: File,
}

impl Repository {
    pub fn new() -> Self {
        let dir = Path::new(REPOSITORY_DIR);
        fs::create_dir_all(dir).unwrap();
        let compressed = create_file("zlib");
        let uncompressed = create_file("txt");
        Self {
            compressed,
            uncompressed,
        }
    }

    pub fn store(&mut self, content: &str) {
        let compressed = compress(content);
        self.compressed.write_all(&compressed).unwrap();
        self.uncompressed.write_all(content.as_bytes()).unwrap();
    }
}

fn create_file(extension: &str) -> File {
    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let dir = Path::new(REPOSITORY_DIR);
    let name = format!("repo-{}.{}", now, extension);
    let path = dir.join(name);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
}

fn compress(content: &str) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(content.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    compressed
}
