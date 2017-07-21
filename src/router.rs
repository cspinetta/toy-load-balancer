extern crate hyper;
extern crate futures;
extern crate redis;

use std::rc::Rc;

use futures::Future;
use futures::stream::Concat2;
use futures::Stream;
use futures::future::ok as FutureOk;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response, Service};
use hyper::client::HttpConnector;
use hyper::error::UriError;
use hyper::Get;
use hyper::header::ContentLength;
use hyper::HttpVersion;
use hyper::Method;
use hyper::Chunk;
use std::cell::RefCell;

use std::sync::Arc;

use hyper::header::{Headers, Host};

use host_resolver::HostResolver;
use cache::Cache;

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

#[derive(Clone, Debug, PartialEq)]
struct RequestHead {
    pub version: HttpVersion,
    pub method: Method,
    pub uri: Uri,
    pub headers: Headers
}

impl RequestHead {
    fn new(version: HttpVersion, method: Method, uri: Uri, headers: Headers) -> RequestHead {
        RequestHead { version, method, uri, headers }
    }

    fn from_request(req: &Request) -> Self {
        Self::new(req.version().clone(), req.method().clone(), req.uri().clone(), req.headers().clone())
    }

    fn with_uri(&self, uri: Uri) -> RequestHead {
        Self::new(self.version.clone(), self.method.clone(), uri, self.headers.clone())
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

    fn create_url(host: &str, uri: Uri) -> Result<Uri, UriError> {
        let query_string = uri.query().map(|str| { format!("?{}", str) }).unwrap_or(String::from(""));
        format!("http://{}{}{}", host, uri.path(), query_string).parse()
    }

    fn req_is_cacheable(req: & Request<Body>) -> bool {
        (*req.method() == Get) // TODO check header Cache: False
    }

    fn cache_key(req: &Request<Body>) -> String {
        req.uri().path().clone().to_string()
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
                        let max_length_cache = cache_ref.clone().max_length();
                        let original_headers = response.headers().clone();
                        let original_status = response.status().clone();

                        let mut is_large = false;
                        let mut content_length: Option<u64> = Option::None;
                        {
                            content_length = response.headers().get::<ContentLength>().map(|length| {length.0});
                        }
                        info!("Content length: {:?}", content_length);
                        let r: Box<Future<Error=hyper::Error, Item=Response>> = match content_length {
                            Some(length) if length <= max_length_cache => {
                                let resp = response
                                    .body()
                                    .concat2()
                                    .map(move |body| {
                                        if original_status == StatusCode::Ok {
                                            cache_ref.clone().set(&cache_key.clone()[..], body.as_ref().clone().to_vec());
                                        }
                                        let resp: Response<Body> = Response::new()
                                            .with_headers(original_headers)
                                            .with_header(ContentLength(body.len() as u64))
                                            .with_status(original_status)
                                            .with_body(body.as_ref().clone().to_vec());
                                        resp
                                    });
                                Box::new(resp)
                            },
                            _ => {
                                let mut res = Response::new();
                                if let Some(len) = response.headers().get::<ContentLength>() {
                                    res.headers_mut().set(len.clone());
                                }
                                res.set_body(response.body());
                                Box::new(FutureOk(res))
                            }
                        };
                        Box::new(r)
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

        let with_new_host = self.uri_with_new_host(&req.uri());
        let head = RequestHead::from_request(&req).with_uri(with_new_host);

        let shared_body: Concat2<Body> = req.body().concat2();

        let r = shared_body.and_then(move |whole_body| {

            let new_req = Self::build_req(&head.clone(), Self::copy_body(&whole_body));

    //        let cloned_req = self.map_req(Self::clone_req(&req));

            info!("Attemp {} for url: {:?}", n_retry, new_req);

            let resp = client_clone.request(new_req).then(move |result| {
                debug!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
                match result {
                    Ok(client_resp) => Box::new(FutureOk(client_resp)),
                    Err(e) => {
                        error!("Connection error: {:?}", &e);
                        if (n_retry < *ref_max.clone()) && (head.method == Get) {

                            let new_req = Self::build_req(&head.clone(), Self::copy_body(&whole_body));
                            self.forward_to_server(&client_clone, new_req, n_retry + 1)
                        } else {
                            Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
                        }

                    }
                }
            });
            Box::new(resp)
        });
        Box::new(r)
    }

    fn build_req(head: &RequestHead, body: Chunk) -> Request {
        let mut new_req = Request::new(head.method.clone(), head.uri.clone());
        new_req.headers_mut().extend(head.headers.iter());
        new_req.set_version(head.version.clone());
        new_req.set_body(body);
        new_req
    }

    fn uri_with_new_host(&self, uri: &Uri) -> Uri {
        let host = self.host_resolver.clone().get_next();
        Self::create_url(host.as_ref(), uri.clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", uri))
    }

    fn copy_body(chunk_ref: &Chunk) -> Chunk {
        Chunk::from(chunk_ref.as_ref().clone().to_vec())
    }

    fn with_new_host(&self, req: &mut Request) {
        let host = self.host_resolver.clone().get_next();
        let uri = Self::create_url(host.as_ref(), req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));
        req.set_uri(uri);
    }

    fn forward_to_server_stream(self, client: &Client<HttpConnector, Body>, mut req: Request<Body>) -> Box<Future<Error=hyper::Error, Item=Response>>
    {

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        self.with_new_host(&mut req);

        info!("Attemp nique stream request for url: {:?}", req.uri());

        let resp = client.request(req).then(move |result| {
            match result {
                Ok(client_resp) => Box::new(FutureOk(client_resp)),
                Err(e) => {
                    error!("Connection error: {:?}", &e);
                    Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
                }
            }
        });
        Box::new(resp)
    }
}
