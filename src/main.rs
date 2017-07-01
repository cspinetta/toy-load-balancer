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

mod proxy;
mod server;

use server::Server;

fn main() {
    pretty_env_logger::init().unwrap();
    start_server();
}
fn start_server() {

    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    info!("Starting Server on {:?}...", addr);

    let mut threads = Vec::new();
    for _ in 0..num_cpus::get() {
        //        let listener = listener.try_clone().unwrap();
        threads.push(thread::spawn(move || {
            let server = Server::new(&addr);
            server.start();
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
