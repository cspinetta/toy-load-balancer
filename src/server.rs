
extern crate net2;
extern crate hyper;

use hyper::server::Http;

use futures::{Stream, Future};

use hyper::Client;
use tokio_core::reactor::Core;

use tokio_core::net::TcpListener;
use std::net::SocketAddr;

use net2::unix::UnixTcpBuilderExt;
use std::sync::Arc;

use router::Proxy;
use router::Router;

pub struct Server<'a> {
    addr: &'a SocketAddr,
    router:  Arc<Router>,
}

impl<'a> Server<'a> {

    pub fn new(addr: &'a SocketAddr, router:  Arc<Router>) -> Server<'a> {
        Server { addr: addr, router: router }
    }

    pub fn start(self) {

        let mut core = Core::new().expect("Create Event Loop");
        let handle = core.handle();
        let service = Proxy::new(Client::new(&handle), self.router.clone());
        let listener = net2::TcpBuilder::new_v4().unwrap()
            .reuse_port(true).unwrap()
            .bind(self.addr).unwrap()
            .listen(128).unwrap();

        let listener = TcpListener::from_listener(listener, self.addr, &handle).unwrap();

        let all_conns = listener.incoming().for_each(|(socket, addr)| {
            Http::new().bind_connection(&handle, socket, addr, service.clone());
            Ok(())
        }).map_err(|err| {
            error!("Error with Tcp Listener: {:?}", err);
        });

        core.run(all_conns).unwrap();
    }
}
