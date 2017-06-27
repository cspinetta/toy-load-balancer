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


fn main() {
    pretty_env_logger::init().unwrap();

    let (pool, join) = TokioPool::new(4)
        .expect("Failed to create event loop");

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let pool: Arc<TokioPool> = Arc::new(pool);
    let pool_ref: Arc<TokioPool> = pool.clone();

    // Use the first pool worker to listen for connections
    pool.next_worker().spawn(move |handle| {

        // Bind a TCP listener to our address
        let listener = TcpListener::bind(&addr, handle).unwrap();

//        let c = service;

        // Listen for incoming clients
        listener.incoming().for_each(move |(socket, addr)| {

            let new_pool_ref = pool_ref.clone();
            let service = Arc::new(Proxy { pool_ref: new_pool_ref.clone() });

            new_pool_ref.next_worker().spawn(move |handle| {
                Arc::new(Http::new()).bind_connection(&handle.clone(), socket, addr, service);
                Ok(())
            });

            Ok(())
        }).map_err(|err| {
            error!("Error with TcpListener: {:?}", err);
        })
    });

    join.join();
}

#[derive(Clone)]
struct Proxy {
    pool_ref: Arc<TokioPool>
}

//fn single_thread() {
//    info!("Starting Load Balancer...");
//    let addr = "127.0.0.1:3000".parse().unwrap();
//    let http: Http<Chunk> = Http::new();
//    let mut core = Core::new().unwrap();
//
//    let handle = core.handle();
//    let listener = TcpListener::bind(&addr, &handle).unwrap();
//    let server = listener.incoming()
//        .for_each(|(sock, addr)| {
//            let service = Proxy { handle: handle.clone() };
//            http.bind_connection(&handle, sock, addr, service);
//            Ok(())
//        });
//    core.run(server).unwrap();
//}

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
        let method = req.method().clone();
        let host = "http://localhost:8000"; // other host
        let uri = self.create_proxy_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut client_req = Request::new(method, uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        info!("Dispatching incoming connection: {:?}", client_req);

        let new_handler = self.pool_ref.next_worker().handle().unwrap().clone();

        let client = Client::new(&new_handler);
        let resp = client.request(client_req)
            .then(move |result| {
                match result {
                    Ok(client_resp) => {
                        Ok(client_resp)
//                        Ok(Response::new()
//                            .with_status(client_resp.status())
//                            .with_headers(client_resp.headers().clone())
//                            .with_body(client_resp.body()))
                    }
                    Err(e) => {
                        error!("{:?}", &e);
                        Err(e)
                    }
                }
            });
        Box::new(resp)
    }
}
