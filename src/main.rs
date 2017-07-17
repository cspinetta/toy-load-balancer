#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate num_cpus;
extern crate net2;

extern crate ipc_channel;

use std::thread;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

mod settings;
mod router;
mod server;
mod host_resolver;
mod cache;

use settings::Settings;
use host_resolver::HostResolver;
use server::Server;
use router::Router;
use std::io::{self, Write};

fn main() {
    pretty_env_logger::init().unwrap();

    info!("Starting server...");

    let settings = Settings::new();

    info!("{:?}", settings);

    start_server(Arc::new(settings));
}

fn start_server(settings: Arc<Settings>) {

    let address = format!("{}:{}", settings.server.host, settings.server.port).parse::<SocketAddr>().unwrap();
    let host_resolver = Arc::new(HostResolver::new(settings.destinations.address.clone()));

    let mut threads = Vec::new();

    let n_threads = num_cpus::get();
    for i in 0..n_threads {
        let settings_ref = settings.clone();
        let host_resolver_ref = host_resolver.clone();
        threads.push(thread::spawn(move || {
            let server = Server::new(&address, host_resolver_ref.clone(), settings_ref.clone());
            server.start();
        }));
    }

    info!("Listening on http://{} with {} threads...", address, n_threads);

    for t in threads {
        t.join().unwrap();
    }
}
