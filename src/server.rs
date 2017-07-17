
extern crate net2;
extern crate hyper;

use settings::Settings;

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

use redis_service::{Cache, RedisCache, NoOpCache};


pub struct Server<'a> {
    addr: &'a SocketAddr,
    host_resolver:  Arc<HostResolver>,
    settings: Arc<Settings>
}

impl<'a> Server<'a> {

    pub fn new(addr: &'a SocketAddr, host_resolver:  Arc<HostResolver>, settings: Arc<Settings>) -> Server<'a> {
        Server { addr: addr, host_resolver: host_resolver, settings: settings }
    }

    pub fn start(self) {
        let mut core = Core::new().expect("Create Event Loop");
        let handle = core.handle();
        let listener = net2::TcpBuilder::new_v4().unwrap()
            .reuse_port(true).unwrap()
            .bind(self.addr).unwrap()
            .listen(128).unwrap();

        let listener = TcpListener::from_listener(listener, self.addr, &handle).unwrap();
        let cache: Arc<Cache> = if self.settings.cache.enable {
            Arc::new(RedisCache::new(self.settings.cache.redis.connection.clone()))
        } else {
            Arc::new(NoOpCache::new())
        };

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
