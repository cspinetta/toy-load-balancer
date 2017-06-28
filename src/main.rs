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

#[derive(Clone)]
struct Proxy {
    pool_ref: Arc<TokioPool>
}

fn server_start_up() {
    let (pool, join) = TokioPool::new(4).expect("Failed to create event loop");

    let pool: Arc<TokioPool> = Arc::new(pool);
    let pool_ref: Arc<TokioPool> = pool.clone();

    let addr = "127.0.0.1:3000".parse().unwrap();
    info!("Starting Load Balancer on {:?}...", addr);
//    let http: Http<Chunk> = Http::new();
    let mut core = Core::new().unwrap();

//    let handle = core.handle();
    let listener = TcpListener::bind(&addr, &core.handle()).unwrap();
    let server = listener
        .incoming()
        .for_each(|(socket, addr)| {
            let service = Arc::new(Proxy { pool_ref: pool_ref.clone() });
            pool_ref.next_worker().spawn(move |handle| {
                Arc::new(Http::new()).bind_connection(&handle.clone(), socket, addr, service);
                Ok(())
            });

            Ok(())
        }).map_err(|err| {
            error!("Error with TcpListener: {:?}", err);
        });
    core.run(server).unwrap();
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

//        let req_ref = Arc::new(req);

        let method = req.method().clone();

        let host = "http://localhost:8000"; // other host
        let uri = self.create_proxy_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut client_req = Request::new(req.method().clone(), uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        self.pool_ref.next_worker().spawn(move |handle| {

            info!("Dispatching incoming connection: {:?}", client_req);

//            let new_handler = self.pool_ref.next_worker().handle().unwrap().clone();

            let client = Client::new(&handle);
            let resp = client.request(client_req).then(move |result| {
                match result {
                    Ok(client_resp) => {
                        info!("Response from client: {:?}", &client_resp);
                        tx.complete(Ok(client_resp));
                        Ok(())
//                        Ok(Response::new()
//                            .with_status(client_resp.status())
//                            .with_headers(client_resp.headers().clone())
//                            .with_body(client_resp.body()))
                    }
                    Err(e) => {
                        error!("{:?}", &e);
                        tx.complete(Err(e));
                        Err("error")
                    }
                }
//                })
//                .then(move |result| {
//                    tx.complete(result);
//                    result
                });
            Ok(())
        });

//        rx.and_then(|client_result| { client_result });

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
