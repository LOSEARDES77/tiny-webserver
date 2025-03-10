mod handler;
mod threadpool;

use crate::handler::RequestHandler;
use crate::threadpool::ThreadPool;
use clap::Parser;
use std::io::ErrorKind;
use std::sync::Arc;
use tiny_http::Server;

fn get_address() -> String {
    if cfg!(debug_assertions) {
        String::from("127.0.0.1")
    } else {
        String::from("0.0.0.0")
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address to use
    #[arg(short, long, default_value_t = get_address())]
    address: String,

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

    let (listener, port) = create_listener(args.address.as_str(), args.port).unwrap();
    println!("Listening on: {}:{}", args.address, port);

    let listner = Arc::new(listener);

    let pool = ThreadPool::new(args.num_workers);

    loop {
        let listener = listner.clone();
        let root = args.root.clone();
        let index_file_name = args.index_file_name.clone();
        let rq = listener.recv();
        pool.execute(move || {
            if let Ok(rq) = rq {
                println!("Recived request: {:?}", rq);
                let handler = RequestHandler::new(root, index_file_name.clone());
                let rp = handler.get_response(&rq);
                println!("Responding with: {:?}", rp.status_code());
                rq.respond(rp).unwrap()
            }
        })
    }
}
