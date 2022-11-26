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
    pub fn open() -> Self {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir).unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let compressed_name = format!("repo-{}.zlib", now);
        let compressed_path = data_dir.join(compressed_name);
        let compressed = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(compressed_path)
            .unwrap();
        let uncompressed_name = format!("repo-{}.txt", now);
        let uncompressed_path = data_dir.join(uncompressed_name);
        let uncompressed = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(uncompressed_path)
            .unwrap();
        Self {
            compressed,
            uncompressed,
        }
    }

    pub fn store(&mut self, content: &str) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(content.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        self.compressed.write_all(&compressed).unwrap();
        self.uncompressed.write_all(content.as_bytes()).unwrap();
    }
}
