use std::collections::HashMap;

#[derive(Debug)]
pub struct Metrics {
    pub log: LogMetrics,
    pub error: ErrorMetrics,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            log: LogMetrics::new(),
            error: ErrorMetrics::new(),
        }
    }
}

#[derive(Debug)]
pub struct LogMetrics {
    pub total_urls: usize,
    pub same_domains: usize,
    pub other_domains: usize,
    pub total_html_files: usize,
    pub compressed_bytes: usize,
    pub uncompressed_bytes: usize,
    pub fetch_count: usize,
    pub process_time: f64,
    pub download_time: f64,
    pub que_size_at_end: usize,
}

impl LogMetrics {
    pub fn new() -> Self {
        Self {
            total_urls: 1,
            same_domains: 1,
            total_html_files: 0,
            compressed_bytes: 0,
            uncompressed_bytes: 0,
            other_domains: 0,
            fetch_count: 0,
            process_time: 0.0,
            download_time: 0.0,
            que_size_at_end: 0,
        }
    }
}

#[derive(Debug)]
pub struct ErrorMetrics {
    pub errors: HashMap<String, usize>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }
}
