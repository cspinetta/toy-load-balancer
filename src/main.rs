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
mod host_resolver;
mod file_utils;

use host_resolver::HostResolver;
use server::Server;
use router::Router;
use file_utils::FileReader;
use std::io::{self, Write};

fn main() {
    pretty_env_logger::init().unwrap();

    info!("Starting server...");

    start_server();
}

fn start_server() {

    let properties = FileReader::read().unwrap();
    let mut availables_servers: Vec<String> = Vec::new();

    let mut addr_value = String::new();

    for i in 0..properties.len(){
        let (property,value) = properties[i].clone();
        if "server=" == property {
            addr_value = value;
//            break;
        } else {
            let s = value.clone();
            availables_servers.push(s);
        }
    }

    let addr = addr_value.parse::<SocketAddr>().unwrap();
    let host_resolver = Arc::new(HostResolver::new(availables_servers));

    let mut threads = Vec::new();

    let n_threads = num_cpus::get();
    for i in 0..n_threads {
        let host_resolver_ref = host_resolver.clone();
        threads.push(thread::spawn(move || {
            let server = Server::new(&addr, host_resolver_ref.clone());
            server.start();
        }));
    }

    info!("Listening on http://{} with {} threads...", addr, n_threads);

    for t in threads {
        t.join().unwrap();
    }
}
