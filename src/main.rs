#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
//extern crate tokio_pool;
extern crate num_cpus;
extern crate net2;

extern crate ipc_channel;

use std::thread;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

mod router;
mod server;
mod duplex;
mod server_manager;

use server_manager::HostResolver;
use server::Server;
use router::Router;
use std::io::{self, Write};

fn main() {
    pretty_env_logger::init().unwrap();

    info!("Starting server...");

    start_server();
}

fn start_server() {

    let addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();
    let i = 0;
    let host_resolver = Arc::new(Mutex::new(HostResolver::new()));

//    let mut channel_vector : Vec<duplex::DuplexStream> = Vec::new();
    let mut threads = Vec::new();

    let n_threads = num_cpus::get();
    for i in 0..n_threads {

//        let (left, right) = duplex::duplex();

//        channel_vector.push (left);

//        let router_ref = router.clone();
        let host_resolver_ref = host_resolver.clone();

        threads.push(thread::spawn(move || {
            let server = Server::new(&addr, host_resolver_ref.clone());
            server.start();
        }));
    }

    info!("Listening on http://{} with {} threads...", addr, n_threads);

//    threads.push(thread::spawn(move || {
//        server_manager::server_manager(channel_vector);
//    }));

    for t in threads {
        t.join().unwrap();
    }
}
