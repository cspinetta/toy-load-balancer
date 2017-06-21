
extern crate hyper;
extern crate futures;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};

use std::ascii::AsciiExt;
use futures::Stream;
use hyper::{Body, Chunk};

use futures::future;
use futures::future::{Either, Map, FutureResult};
use futures::stream::Concat;
use futures::Future;


fn main() {
    println!("Hello, world!");
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(LoadBalancingHandler)).unwrap();
    server.run().unwrap();
}

const PHRASE: &'static str = "Hello, World!";

struct LoadBalancingHandler;

impl Service for LoadBalancingHandler {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Either<
        FutureResult<Self::Response, Self::Error>,
        Map<Concat<Body>, fn(Chunk) -> Self::Response>
    >;

    fn call(&self, req: Self::Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&Method::Get, "/health-check") => {

                let response = Response::new()
                        .with_header(ContentLength(PHRASE.len() as u64))
                        .with_body(PHRASE);
                Either::A(futures::future::ok(response))
            },
            (&Method::Post, "/echo") => {
                Either::B(req.body()
                             .concat()
                             .map(reverse))
            },
            _ => {
                Either::A(future::ok(Response::new().with_status(StatusCode::NotFound)))
            }
        }
    }
}

fn reverse(chunk: Chunk) -> Response {
    let reversed = chunk.iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>();
    Response::new()
        .with_body(reversed)
}

//fn to_uppercase(chunk: Chunk) -> Chunk {
//    let uppered = chunk.iter()
//        .map(|byte| byte.to_ascii_uppercase())
//        .collect::<Vec<u8>>();
//    Chunk::from(uppered)
//}


//#[macro_use]
//extern crate serde_derive;
//
//extern crate futures_cpupool;
//extern crate tokio_proto;
//extern crate tokio_minihttp;
//extern crate futures;
//extern crate rand;
//extern crate serde;
//extern crate serde_json;
//extern crate tokio_service;
//
//use std::io;
//
//use futures::{BoxFuture, Future};
//use futures_cpupool::CpuPool;
//use rand::Rng;
//use tokio_minihttp::{Request, Response};
//use tokio_proto::TcpServer;
//use tokio_service::Service;

//fn main() {
////    println!("Hello, world!");
////
////    let addr = "0.0.0.0:8080".parse().unwrap();
////
////    let thread_pool = CpuPool::new(10);
////
////    TcpServer::new(tokio_minihttp::Http, addr).serve(move || {
////        Ok(Server {
////            thread_pool: thread_pool.clone(),
////        })
////    })
//}

//struct Server {
//    thread_pool: CpuPool
//}
//
//impl Service for Server {
//    type Error = io::Error;
//    type Future = BoxFuture<Response, io::Error>;
//    type Request = Request;
//    type Response = Response;
//
//    fn call(&self, req: Self::Request) -> Self::Future {
//        assert_eq!(req.path(), "/db");
//        let random_id = rand::thread_rng().gen_range(1, 5);
//        unimplemented!()
//    }
//}
//
//#[derive(Serialize)]
//struct Message {
//    id: i32,
//    body: String,
//}
