
extern crate net2;
extern crate hyper;

use futures::{Stream, Future};

use hyper::server::Http;
use hyper::Client;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use net2::unix::UnixTcpBuilderExt;
use std::sync::Arc;
use std::net::SocketAddr;

use router::Proxy;
use host_resolver::HostResolver;

use redis_service::Cache;


pub struct Server<'a> {
    addr: &'a SocketAddr,
    host_resolver:  Arc<HostResolver>,
}

impl<'a> Server<'a> {

    pub fn new(addr: &'a SocketAddr, host_resolver:  Arc<HostResolver>) -> Server<'a> {
        Server { addr: addr, host_resolver: host_resolver }
    }

    pub fn start(self) {
        let mut core = Core::new().expect("Create Event Loop");
        let handle = core.handle();
        let listener = net2::TcpBuilder::new_v4().unwrap()
            .reuse_port(true).unwrap()
            .bind(self.addr).unwrap()
            .listen(128).unwrap();

        let listener = TcpListener::from_listener(listener, self.addr, &handle).unwrap();
        let cache = Arc::new(Cache::new());

        let all_conns = listener.incoming().for_each(|(socket, addr)| {
            let service = Proxy::new(Client::new(&handle), self.host_resolver.clone(), cache.clone());
            Http::new().bind_connection(&handle, socket, addr, service.clone());
            Ok(())
        }).map_err(|err| {
            error!("Error with Tcp Listener: {:?}", err);
        });

        core.run(all_conns).unwrap();
    }
}
