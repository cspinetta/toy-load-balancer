extern crate hyper;
extern crate futures;

use futures::Future;
use futures::future::ok as FutureOk;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response, Service};
use hyper::client::HttpConnector;
use hyper::error::UriError;
use hyper::Get;

use std::sync::{Arc, Mutex};
use std::io::{self, Write};

use server_manager::HostResolver;

#[derive(Clone)]
pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub host_resolver: Arc<Mutex<HostResolver>>,
}

impl Proxy {

    pub fn new(client: Client<HttpConnector, Body>, host_resolver: Arc<Mutex<HostResolver>>) -> Proxy {
        Proxy { client, host_resolver }
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        info!("Dispatching request: {:?}", &req);
        Router::new(self.host_resolver.clone()).dispatch_request(&self.client, req, 1)
    }
}

#[derive(Clone)]
pub struct Router {
    max_retry: Arc<u32>,
    host_resolver: Arc<Mutex<HostResolver>>
}

impl Router {

    pub fn new(host_resolver: Arc<Mutex<HostResolver>>) -> Router {
        Router { max_retry: Arc::new(3), host_resolver: host_resolver }
    }

    fn clone_req(req: &Request) -> Request {
        let mut new_req = Request::new(req.method().clone(), req.uri().clone());
        new_req.headers_mut().extend(req.headers().iter());
        new_req
    }

    fn clone_req_custom_uri(req: &Request, uri: &Uri) -> Request {
        let mut new_req = Request::new(req.method().clone(), uri.clone());
        new_req.headers_mut().extend(req.headers().iter());
        new_req
    }

    fn create_url(host: &str, uri: Uri) -> Result<Uri, UriError> {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }

    fn map_req(&self, req: Request) -> Request {
        let host = self.host_resolver.lock().unwrap().get_next();
        let uri = Self::create_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));
        Self::clone_req_custom_uri(&req, &uri)
    }

    fn dispatch_request(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {
        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        let cloned_req = self.map_req(Self::clone_req(&req));

        info!("Attemp {} for url: {:?}", n_retry, cloned_req);

        let resp = client.request(Self::clone_req(&cloned_req)).then(move |result| {
            debug!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
            match result {
                Ok(client_resp) => {
                    if client_resp.status() == hyper::StatusCode::Ok {
                        Box::new(FutureOk(client_resp))
                    } else if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }
                },
                Err(e) => {
                    error!("Connection error: {:?}", &e);
                    if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }

                }
            }
        });
        Box::new(resp)
    }
}
