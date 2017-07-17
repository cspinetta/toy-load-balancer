extern crate hyper;
extern crate futures;
extern crate redis;

use futures::Future;
use futures::Stream;
use futures::future::ok as FutureOk;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response, Service};
use hyper::client::HttpConnector;
use hyper::error::UriError;
use hyper::Get;
use hyper::header::ContentLength;
use std::sync::Arc;

use host_resolver::HostResolver;
use redis_service::Cache;

#[derive(Clone)]
pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub host_resolver: Arc<HostResolver>,
    pub cache: Arc<Cache>,
}

impl Proxy {

    pub fn new(client: Client<HttpConnector, Body>, host_resolver: Arc<HostResolver>, cache: Arc<Cache>) -> Proxy {
        Proxy { client, host_resolver, cache }
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        info!("Dispatching request: {:?}", &req);
        Router::new(self.host_resolver.clone(), self.cache.clone())
            .dispatch_request(&self.client, req)
    }
}

#[derive(Clone)]
pub struct Router {
    max_retry: Arc<u32>,
    host_resolver: Arc<HostResolver>,
    cache: Arc<Cache>
}

impl Router {

    pub fn new(host_resolver: Arc<HostResolver>, cache: Arc<Cache>) -> Router {
        Router { max_retry: Arc::new(3), host_resolver: host_resolver, cache: cache }
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
        format!("http://{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }

    fn req_is_cacheable(req: & Request<Body>) -> bool {
        (*req.method() == Get) // TODO check header Cache: False
    }

    fn cache_key(req: &Request<Body>) -> String {
        req.uri().path().clone().to_string()
    }

    fn map_req(&self, req: Request) -> Request {
        let host = self.host_resolver.clone().get_next();
        let uri = Self::create_url(host.as_ref(), req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));
        Self::clone_req_custom_uri(&req, &uri)
    }

    fn dispatch_request(self, client: &Client<HttpConnector, Body>, req: Request<Body>) -> Box<Future<Error=hyper::Error, Item=Response>> {
        if Self::req_is_cacheable(&req) {
            self.with_cache(client, req)
        } else {
            self.forward_to_server(client, req, 1)
        }
    }

    fn with_cache(self, client: &Client<HttpConnector, Body>, req: Request<Body>) -> Box<Future<Error=hyper::Error, Item=Response>> {
        let cache_key = Self::cache_key(&req);
        let cache_ref: Arc<Cache> = self.cache.clone();
        let cache_response = cache_ref.clone().get(&cache_key[..]);
        let resp: Box<Future<Error=hyper::Error, Item=Response>> = match cache_response {
            Ok(response) =>  {
                info!("Response from Redis: {:?}", response);
                Box::new(FutureOk(Response::new().with_body(response)))
            },
            Err(e) => {
                let resp = self
                    .forward_to_server(client, req, 1)
                    .and_then(move |response| {
                        let original_headers = response.headers().clone();
                        let original_status = response.status().clone();
                        let resp = response
                            .body()
                            .fold(Vec::new(), |mut acc, chunk| {
                                acc.extend_from_slice(&*chunk);
                                futures::future::ok::<_, hyper::Error>(acc)
                            })
                            .map(move |body| {
                                if original_status == StatusCode::Ok {
                                    cache_ref.clone().set(&cache_key.clone()[..], String::from_utf8(body.clone()).unwrap());
                                }
                                let body_str = String::from_utf8(body).unwrap();
                                let resp: Response<Body> = Response::new()
                                    .with_headers(original_headers)
                                    .with_header(ContentLength(body_str.len() as u64))
                                    .with_status(original_status)
                                    .with_body(body_str.clone());
                                resp
                            });
                        Box::new(resp)
                    });
                Box::new(resp)
            }
        };
        resp
    }

    fn forward_to_server(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        let cloned_req = self.map_req(Self::clone_req(&req));

        info!("Attemp {} for url: {:?}", n_retry, cloned_req);

        let resp = client.request(Self::clone_req(&cloned_req)).then(move |result| {
            debug!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
            match result {
                Ok(client_resp) => Box::new(FutureOk(client_resp)),
                Err(e) => {
                    error!("Connection error: {:?}", &e);
                    if (n_retry < *ref_max.clone()) && (*cloned_req.method() == Get) {
                        self.forward_to_server(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }

                }
            }
        });
        Box::new(resp)
    }
}
