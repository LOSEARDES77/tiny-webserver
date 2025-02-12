use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tiny_http::{Header, Request, Response};
#[derive(Debug, Clone)]
pub struct RequestHandler {
    root: PathBuf,
    index_file: String,
}

impl RequestHandler {
    pub fn new(root: String, index_file: String) -> Self {
        let root = PathBuf::from(root).canonicalize().unwrap(); // Ensure absolute path
        RequestHandler { root, index_file }
    }

    pub fn get_response(&self, rq: &Request) -> Response<Cursor<Vec<u8>>> {
        let url = rq.url().trim_start_matches('/'); // Remove leading `/`
        let requested_path = self.clone().root.join(url); // Construct full path

        if let Some(rp) = self.clone().get_response_from_file(&requested_path) {
            return rp;
        }

        Response::from_string(include_str!("../assets/404.html"))
            .with_status_code(404)
            .with_header(Header::from_str("Content-Type: text/html").unwrap())
    }

    fn get_response_from_file(self, requested_path: &Path) -> Option<Response<Cursor<Vec<u8>>>> {
        if let Ok(real_path) = requested_path.canonicalize() {
            // Security check: ensure path is within root directory
            if !real_path.starts_with(&self.root) {
                return None;
            }

            if real_path.is_dir() {
                let index_path = real_path.join(&self.index_file);

                if index_path.exists() && index_path.is_file() {
                    // Serve the index file if it exists
                    if let Ok(mut file) = File::open(index_path) {
                        let mut buf = String::new();
                        file.read_to_string(&mut buf).unwrap();
                        let content_type = match real_path.extension().and_then(|e| e.to_str()) {
                            Some("html") => "text/html",
                            Some("css") => "text/css",
                            Some("js") => "text/javascript",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("svg") => "image/svg+xml",
                            Some("pdf") => "application/pdf",
                            _ => "text/html",
                        };
                        return Some(Response::from_data(buf).with_header(
                            Header::from_str(&format!("Content-Type: {}", content_type)).unwrap(),
                        ));
                    }
                } else {
                    // No index file, show directory listing
                    return Some(self.explorer(real_path));
                }
            } else if real_path.is_file() {
                // Handle regular files
                if let Ok(mut file) = File::open(&real_path) {
                    let mut buf = Vec::new();
                    if file.read_to_end(&mut buf).is_ok() {
                        // Determine content type based on file extension
                        let content_type = match real_path.extension().and_then(|e| e.to_str()) {
                            Some("html") => "text/html",
                            Some("css") => "text/css",
                            Some("js") => "text/javascript",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("svg") => "image/svg+xml",
                            Some("pdf") => "application/pdf",
                            _ => "text/html",
                        };

                        return Some(Response::from_data(buf).with_header(
                            Header::from_str(&format!("Content-Type: {}", content_type)).unwrap(),
                        ));
                    }
                }
            }
        }
        None
    }

    pub fn explorer(&self, dir: PathBuf) -> Response<Cursor<Vec<u8>>> {
        let mut entries = Vec::new();

        let relative_path = dir
            .strip_prefix(&self.root)
            .unwrap_or(&PathBuf::new())
            .display()
            .to_string();

        // Add parent directory link if not at root
        if dir != self.root {
            // Get parent path by removing last component from current path
            let parent_path = if relative_path.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", relative_path)
                    .rsplit('/')
                    .nth(1)
                    .map(|s| if s.is_empty() { "/" } else { s })
                    .unwrap_or("/")
                    .to_string()
            };

            entries.push(format!(
                r#"<li class="file-item">
                    <a href="{}">
                        <svg class="file-icon" viewBox="0 0 20 20">
                            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                        </svg>
                        ..
                    </a>
                </li>"#,
                parent_path
            ));
        }

        if let Ok(dir_entries) = fs::read_dir(&dir) {
            for entry in dir_entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = metadata.is_dir();
                    let size = if !is_dir {
                        format!("{:.1} KB", metadata.len() as f64 / 1024.0)
                    } else {
                        String::new()
                    };

                    let icon = if is_dir {
                        r#"<path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>"#
                    } else {
                        r#"<path d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z"/>"#
                    };

                    // Construct absolute path for href
                    let href = if relative_path.is_empty() {
                        format!("/{}", name)
                    } else {
                        format!("/{}/{}", relative_path, name)
                    };

                    entries.push(format!(
                        r#"<li class="file-item">
                        <a href="{}">
                            <svg class="file-icon" viewBox="0 0 20 20">
                                {}
                            </svg>
                            {}
                        </a>
                        <span class="file-info">
                            <span class="file-size">{}</span>
                        </span>
                    </li>"#,
                        href, icon, name, size
                    ));
                }
            }
        }

        let html = include_str!("../assets/file_explorer.html")
            .replace("{cwd}", &format!("/{}", relative_path))
            .replace("{list}", &entries.join("\n"));

        Response::from_string(html)
            .with_header(Header::from_str("Content-Type: text/html").unwrap())
    }
}
