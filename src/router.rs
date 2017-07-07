extern crate hyper;
extern crate futures;

use futures::Future;
use futures::future::FutureResult;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response, Service};
use hyper::client::HttpConnector;
use hyper::error::UriError;
use hyper::Get;

use std::sync::Arc;
use std::io::{self, Write};

#[derive(Clone)]
pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub router: Arc<Router>,
    pub redireccion: String,
}

impl Proxy {

    pub fn new(client: Client<HttpConnector, Body>, router: Arc<Router>, redireccion: String) -> Proxy {
        Proxy { client, router, redireccion }
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
//        let ref_router = self.router.clone();
        info!("Dispatching request: {:?}", &req);
        Router::new().dispatch_request(&self.client, req, 1)
    }
}

#[derive(Clone)]
pub struct Router {
    max_retry: Arc<u32>
}

impl Router {

    pub fn new() -> Router {
        Router { max_retry: Arc::new(3) }
    }

    fn clone_req(req: &Request) -> Request {
        let mut new_req = Request::new(req.method().clone(), req.uri().clone());
        new_req.headers_mut().extend(req.headers().iter());
        new_req
    }

    fn create_url(host: &str, uri: Uri) -> Result<Uri, UriError> {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }

    fn map_req(req: Request) -> Request {
        let host = "http://localhost:3001"; // other host
        let uri = Self::create_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));
        Self::clone_req(&req)
    }

    fn dispatch_request(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {
        info!("Attemp {}", n_retry);

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        let cloned_req = Self::map_req(Self::clone_req(&req));

        let resp = client.request(Self::clone_req(&cloned_req)).then(move |result| {
            debug!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
            match result {
                Ok(client_resp) => {
                    if client_resp.status() == hyper::StatusCode::Ok {
                        Box::new(futures::future::ok(client_resp))
                    } else if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }
                },
                Err(e) => {
                    error!("Connection error: {:?}", &e);
                    if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }

                }
            }
        });
        Box::new(resp)
    }
}
