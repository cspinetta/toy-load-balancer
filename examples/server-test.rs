#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
//extern crate tokio_pool;
extern crate num_cpus;

use std::{iter, env};

use futures::future::FutureResult;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

extern crate net2;
use std::thread;
use std::net::SocketAddr;
use std::sync::Arc;
use std::str;

fn main() {
    pretty_env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    info!("Arguments supplied: {:?}", args[1]);
    if args.len() < 1 {
        panic!("Please supply address to listen connections");
    }
    let addr = args[1].parse::<SocketAddr>().expect("Parse argument 0 as addr::SocketAddr");

    start_server(addr);
}

struct PayloadGenerator;
impl PayloadGenerator {
    fn generate_string(n: usize) -> Vec<u8> {
        iter::repeat('a').take(n).map(|x| x as u8).collect::<Vec<u8>>()
    }
    fn medium_string() -> Vec<u8> {
        Self::generate_string(20 * 1024) // 20 kb
    }
    fn large_string() -> Vec<u8> {
        Self::generate_string(200 * 1024) // 200 kb
    }
}


fn start_server(addr: SocketAddr) {

    let service = ServiceHandler::new();

    let server = Http::new().bind(&addr, move || { Ok(service.clone()) }).unwrap();
    info!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}

static PONG: &'static [u8] = b"Pong";

#[derive(Clone)]
struct ServiceHandler;

impl ServiceHandler {

    fn new() -> ServiceHandler {
        ServiceHandler {  }
    }

    fn make_response(payload: Vec<u8>) -> Response {
        Response::new()
            .with_header(ContentLength(payload.len() as u64))
            .with_body(payload)
    }
}

impl Service for ServiceHandler {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let response = match (req.method(), req.path()) {
            (&Get, "/") => {
                Response::new()
            },
            (&Get, "/ping") => {
                Response::new()
                    .with_header(ContentLength(PONG.len() as u64))
                    .with_body(PONG)
            },
            (&Get, "/medium-payload") => {
                Self::make_response(PayloadGenerator::medium_string())
            },
            (&Get, "/large-payload") => {
                Self::make_response(PayloadGenerator::large_string())
            },
//            (&Get, "/custom-payload") => {
//                Self::make_response(PayloadGenerator::large_string())
//            },
            (&Post, "/echo") => {
                let mut res = Response::new();
                if let Some(len) = req.headers().get::<ContentLength>() {
                    res.headers_mut().set(len.clone());
                }
                res.with_body(req.body())
            },
            _ => {
                Response::new()
                    .with_status(StatusCode::NotFound)
            }
        };

        futures::future::ok(response)
    }
}
