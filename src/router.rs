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
        format!("http://{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
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
                        let original_headers = response.headers().clone();
                        let original_status = response.status().clone();
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
                    });
                Box::new(resp)
            }
        };
        resp
    }

    fn clone_req_without_body(req: &Request) -> Request {
        let mut new_req = Request::new(req.method().clone(), req.uri().clone());
        new_req.headers_mut().extend(req.headers().iter());
        new_req.set_version(req.version());
        new_req
    }

    fn head_of_req(req: &Request) -> RequestHead {
        RequestHead {
            method: req.method().clone(),
            uri: req.uri().clone(),
            headers: req.headers().clone(),
            version: req.version().clone(),
        }
    }

    fn clone_req_custom_uri(req: &Request, uri: &Uri) -> Request {
        let mut new_req = Request::new(req.method().clone(), uri.clone());
        new_req.headers_mut().extend(req.headers().iter());
        new_req
    }

    fn map_req(&self, req: Request) -> Request {
        let host = self.host_resolver.clone().get_next();
        let uri = Self::create_url(host.as_ref(), req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));
        Self::clone_req_custom_uri(&req, &uri)
    }

    fn forward_to_server(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        let mut head = Self::head_of_req(&req);
        let with_new_host = self.uri_with_new_host(&head.uri);
        head.uri = with_new_host;
//        let shared_body = Rc::new(req.body()
//            .fold(Vec::new(), |mut acc, chunk| {
//                acc.extend_from_slice(&*chunk);
//                futures::future::ok::<_, hyper::Error>(acc)
//            }));
        let shared_body: Concat2<Body> = req.body().concat2();

        let r = shared_body.and_then(move |chunk| {

            let new_req = Self::build_req(&head.clone(), chunk);

    //        let cloned_req = self.map_req(Self::clone_req(&req));

            info!("Attemp {} for url: {:?}", n_retry, new_req);

            let resp = client_clone.clone().request(new_req).then(move |result| {
                debug!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
                match result {
                    Ok(client_resp) => Box::new(FutureOk(client_resp)),
                    Err(e) => {
                        error!("Connection error: {:?}", &e);
    //                    if (n_retry < *ref_max.clone()) && (*cloned_req.method() == Get) {
    //                        self.forward_to_server(&client_clone, Self::clone_req(&cloned_req), n_retry + 1)
    //                    } else {
                            Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
    //                    }

                    }
                }
            });
            Box::new(resp)
//            Box::new(FutureOk(Response::new().with_status(StatusCode::ServiceUnavailable)))
        });
        Box::new(r)
//        r
    }

    fn build_req(head: &RequestHead, body: Chunk) -> Request {
        let mut new_req = Request::new(head.method.clone(), head.uri.clone());
        new_req.headers_mut().extend(head.headers.iter());
        new_req.set_version(head.version.clone());

//        let all_body = body.iter()
//            .cloned()
//            .collect::<Vec<u8>>();
        new_req.set_body(body);
        new_req
    }

    fn uri_with_new_host(&self, uri: &Uri) -> Uri {
        let host = self.host_resolver.clone().get_next();
        Self::create_url(host.as_ref(), uri.clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", uri))
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
