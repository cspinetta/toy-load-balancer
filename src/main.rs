
extern crate hyper;
extern crate futures;
extern crate tokio_core;

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


fn main() {
    println!("Starting Load Balancer...");

    let addr = "127.0.0.1:3000".parse().unwrap();

    let http = Http::new();
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let listener = TcpListener::bind(&addr, &handle).unwrap();

    let server = listener.incoming()
        .for_each(|(sock, addr)| {
            let service = Proxy { handle: handle.clone() };
            http.bind_connection(&handle, sock, addr, service);
            Ok(())
        });

    core.run(server).unwrap();
}

struct Proxy {
    handle: Handle
}

impl Proxy {

    fn create_proxy_url(&self, host: &str, uri: Uri) -> Uri {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse().unwrap()
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let method = req.method().clone();
        let uri = self.create_proxy_url("http://reddit.com", req.uri().clone());

        let mut client_req = Request::new(method, uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        println!("Request: {:?}", client_req);

        let client = Client::new(&self.handle);
        let resp = client.request(client_req)
            .then(move |result| {
                match result {
                    Ok(client_resp) => {
                        Ok(Response::new()
                            .with_status(client_resp.status())
                            .with_headers(client_resp.headers().clone())
                            .with_body(client_resp.body()))
                    }
                    Err(e) => {
                        println!("{:?}", &e);
                        Err(e)
                    }
                }
            });
        Box::new(resp)
    }
}
