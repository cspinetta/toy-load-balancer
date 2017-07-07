
extern crate net2;
extern crate hyper;

use duplex;

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
use std::time::Duration;
//use std::time;

pub struct Server<'a> {
    addr: &'a SocketAddr,
    router:  Arc<Router>,
    channel: duplex::DuplexStream,
    id: usize,
}

impl<'a> Server<'a> {

    pub fn new(addr: &'a SocketAddr, router:  Arc<Router>, channel: duplex::DuplexStream, id: usize) -> Server<'a> {
        Server { addr: addr, router: router, channel: channel, id: id }
    }

    pub fn start(self) {

        //let mut timer = time::new().unwrap();
        let mut core = Core::new().expect("Create Event Loop");
        let handle = core.handle();
        //let service = Proxy::new(Client::new(&handle), self.router.clone());
        let listener = net2::TcpBuilder::new_v4().unwrap()
            .reuse_port(true).unwrap()
            .bind(self.addr).unwrap()
            .listen(128).unwrap();

        let listener = TcpListener::from_listener(listener, self.addr, &handle).unwrap();

        let all_conns = listener.incoming().for_each(|(socket, addr)| {

            self.channel.tx.send(self.id.to_string());

            let redireccion = self.channel.rx.recv_timeout(Duration::from_millis(50)).unwrap();

            if (redireccion.is_empty() || redireccion == ""){

                //salir con error
            }

            //self.channel.tx.send(redireccion.clone());

            let service = Proxy::new(Client::new(&handle), self.router.clone(), redireccion);

            Http::new().bind_connection(&handle, socket, addr, service.clone());
            Ok(())
        }).map_err(|err| {
            error!("Error with Tcp Listener: {:?}", err);
        });

        core.run(all_conns).unwrap();
    }
}
