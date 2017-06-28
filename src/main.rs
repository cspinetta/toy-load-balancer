#[macro_use] extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate tokio_pool;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};

use std::ascii::AsciiExt;
use futures::{Stream, Future};
use hyper::{Body, Chunk};

use futures::future;
use futures::future::{Either, Map, FutureResult};
use futures::stream::Concat2;

use std::io::{self, Write};
use hyper::Client;
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;

use tokio_core::net::TcpListener;

use hyper::Uri;
use hyper::error::UriError;

use tokio_pool::TokioPool;
use std::sync::Arc;

use std::net::SocketAddr;
use std::cell::RefCell;
use std::mem;
use std::borrow::Borrow;

use futures::sync::oneshot;


fn main() {
    pretty_env_logger::init().unwrap();
    server_start_up();
}

fn server_start_up() {
    let (pool, join) = TokioPool::new(4).expect("Failed to create event loop");

    let pool: Arc<TokioPool> = Arc::new(pool);
    let pool_ref: Arc<TokioPool> = pool.clone();

    let addr = "127.0.0.1:3000".parse().unwrap();
    info!("Starting Load Balancer on {:?}...", addr);
    let mut server_event_loop = Core::new().expect("Create Server Event Loop");
    let mut client_event_loop = Arc::new(Core::new().expect("Create Client Event Loop"));

    let mut client_event_loop_ref = client_event_loop.clone();

    let handle = server_event_loop.handle();
    let handle_ref = server_event_loop.handle();

    let listener = TcpListener::bind(&addr, &handle).unwrap();
    let server = listener
        .incoming()
        .for_each(|(socket, addr)| {
            let service = Proxy { };
            Http::new().bind_connection(&handle_ref.clone(), socket, addr, service);
            Ok(())
        }).map_err(|err| {
            error!("Error with TcpListener: {:?}", err);
        });

    server_event_loop.run(server).unwrap();
}

struct Proxy {
//    client_event_loop: Core
}

impl Proxy {

    fn create_proxy_url(&self, host: &str, uri: Uri) -> Result<Uri, UriError> {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let (tx, rx) = oneshot::channel();

        let mut client_event_loop = Core::new().expect("Create Client Event Loop");

        let host = "http://localhost:8000"; // other host
        let uri = self.create_proxy_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut client_req = Request::new(req.method().clone(), uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        info!("Dispatching incoming connection: {:?}", client_req);

        let handle = client_event_loop.handle().clone();

        let client = Client::new(&handle);

        let resp = client.request(client_req).then(move |result| {
            match result {
                Ok(client_resp) => {
                    info!("Response from client: {:?}", &client_resp);
                    tx.send(Ok(client_resp));
                    Ok(())
                }
                Err(e) => {
                    error!("{:?}", &e);
                    tx.send(Err(e));
                    Err(())
                }
            }
        });
        info!("it's here!!!!!");
        client_event_loop.run(resp);

        rx
            .then(|result| {
                match result {
                    Ok(f) => f,
                    Err(canceled) => {
                        error!("Client canceled: {:?}", &canceled);
                        Ok(Response::new())
                    }
                }
            })
            .boxed()
    }
}
