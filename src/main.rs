#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
//extern crate tokio_pool;
extern crate num_cpus;
extern crate net2;

use std::thread;
use std::net::SocketAddr;
use std::sync::Arc;

mod router;
mod server;
mod duplex;
mod server_manager;
mod file_utils;

use server::Server;
use router::Router;
use std::io::{self, Write};

fn main() {
    pretty_env_logger::init().unwrap();

    println!("starting server");
    io::stdout().flush().expect("flushed");

    start_server();
}

fn start_server() {

    let addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();
    let i = 0;
    let router = Arc::new(Router::new());

    let mut channel_vector : Vec<duplex::DuplexStream> = Vec::new();
    let mut threads = Vec::new();

    let n_threads = num_cpus::get();
    for i in 0..n_threads {

        let (left, right) = duplex::duplex();

        channel_vector.push (left);

        let router_ref = router.clone();

        threads.push(thread::spawn(move || {
            let server = Server::new(&addr, router_ref.clone(), right, i);
            server.start();
        }));
    }

    println!("Listening on http://{} with {} threads...", addr, n_threads);
    io::stdout().flush().expect("flushed");
    info!("Listening on http://{} with {} threads...", addr, n_threads);

    threads.push(thread::spawn(move || {
        server_manager::server_manager(channel_vector);
    }));

    for t in threads {
        t.join().unwrap();
    }
}
