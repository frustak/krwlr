use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

pub struct Repository {
    pub compressed: File,
    pub uncompressed: File,
}

impl Repository {
    pub fn new() -> Self {
        let dir = Path::new("./data");
        fs::create_dir_all(dir).unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let compressed_name = format!("repo-{}.zlib", now);
        let compressed_path = dir.join(compressed_name);
        let compressed = create_file(&compressed_path);
        let uncompressed_name = format!("repo-{}.txt", now);
        let uncompressed_path = dir.join(uncompressed_name);
        let uncompressed = create_file(&uncompressed_path);
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

fn create_file(path: &Path) -> File {
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
