mod fmt;
mod handler;
mod logger;
mod threadpool;

use crate::handler::RequestHandler;
use crate::threadpool::ThreadPool;
use clap::Parser;
use logger::{log_request, log_response};
use std::env::args;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Instant;
use tiny_http::Server;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address to use (default: 127.0.0.1)
    #[arg(short, long)]
    address: Option<String>,

    /// Expose to the public internet (Sets address to 0.0.0.0)
    #[arg(short, long)]
    expose: bool,

    /// Port to use
    #[arg(short, long, default_value_t = 80)]
    port: u16,

    /// Number of concurrent workers for handling requests
    #[arg(short, long, default_value_t = 0)]
    num_workers: usize,

    /// File to serve by default in a directory
    #[arg(short, long, default_value_t = String::from("index.html"))]
    index_file_name: String,

    /// Root directory to use
    #[arg(short, long, default_value_t = String::from("."),)]
    root: String,

    /// Show request headers
    #[arg(short = 'H', long, default_value_t = false)]
    show_headers: bool,
}

fn create_listener(host: &str, port: u16) -> Result<(Server, u16), std::io::Error> {
    let address = format!("{}:{}", host, port);
    match Server::http(&address) {
        Ok(listener) => Ok((listener, port)),
        Err(e) => {
            let e = e.downcast::<std::io::Error>().unwrap();
            match e.kind() {
                ErrorKind::AddrInUse => {
                    println!("Port {} in use, trying {}", port, port + 1);
                    create_listener(host, 1 + port)
                }
                ErrorKind::PermissionDenied => {
                    println!(
                        "{}\nError: Could not open port {} due to insufficient permission",
                        port, e
                    );
                    #[cfg(target_os = "linux")]
                    println!("tip: try running \"sudo setcap cap_net_bind_service=+ep {}\" to add permission or run it as sudo", args().collect::<Vec<String>>()[0]);
                    println!("Trying with port 8000");
                    create_listener(host, 8000)
                }
                _ => Err(*e),
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let addr;
    if let Some(address) = args.address {
        addr = address;
    } else if args.expose {
        addr = "0.0.0.0".to_string();
    } else {
        addr = "127.0.0.1".to_string();
    }

    let (listener, port) = create_listener(&addr, args.port).unwrap();
    println!("Web server listening at http://{}:{}/", addr, port);

    let listener = Arc::new(listener);

    let pool = ThreadPool::new(args.num_workers);

    loop {
        let listener = listener.clone();
        let root = args.root.clone();
        let index_file_name = args.index_file_name.clone();
        let rq = listener.recv();
        pool.execute(move || {
            if let Ok(rq) = rq {
                let start_time = Instant::now();
                log_request(&rq);
                if args.show_headers {
                    println!("--------------------");
                    for header in rq.headers() {
                        println!("{}: {}", header.field, header.value);
                    }
                    println!("--------------------");
                }
                let handler = RequestHandler::new(root, index_file_name.clone());
                let rp = handler.get_response(&rq);
                log_response(&rq, &rp, start_time.elapsed());
                rq.respond(rp).unwrap();
            }
        })
    }
}
