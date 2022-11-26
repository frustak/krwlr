#[derive(Debug)]
pub struct Metrics {
    pub total_urls: usize,
    pub other_domains: usize,
    pub same_domains: usize,
    pub total_html_files: usize,
    pub downloaded_bytes: usize,
    pub compressed_bytes: usize,
    pub uncompressed_bytes: usize,
    pub fetch_count: usize,
    pub process_time: f64,
    pub download_time: f64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_urls: 1,
            total_html_files: 0,
            same_domains: 1,
            downloaded_bytes: 0,
            compressed_bytes: 0,
            uncompressed_bytes: 0,
            other_domains: 0,
            fetch_count: 0,
            process_time: 0.0,
            download_time: 0.0,
        }
    }
}
