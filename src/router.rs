extern crate hyper;
extern crate futures;

use hyper::server::{Request, Response, Service};
use hyper::StatusCode;

use futures::Future;
use hyper::Body;

use hyper::Client;

use hyper::Uri;
use hyper::error::UriError;
use hyper::client::HttpConnector;

use std::sync::Arc;

#[derive(Clone)]
pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub router: Arc<Router>,
}

impl Proxy {

    pub fn new(client: Client<HttpConnector, Body>, router: Arc<Router>) -> Proxy {
        Proxy { client, router }
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let forwarded_req = self.router.map_req(req);

        info!("Dispatching request: {:?}", forwarded_req);

        let resp = self.client.request(forwarded_req).then(move |result| {
            let response = match result {
                Ok(client_resp) => client_resp,
                Err(e) => {
                    error!("{:?}", &e);
                    Response::new().with_status(StatusCode::ServiceUnavailable)
                }
            };
            futures::future::ok(response)
        });

        Box::new(resp) as Self::Future
    }
}

#[derive(Clone)]
pub struct Router {

}

impl Router {

    pub fn new() -> Router {
        Router {}
    }

    fn create_url(&self, host: &str, uri: Uri) -> Result<Uri, UriError> {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }

    pub fn map_req(&self, req: Request) -> Request {

        let host = "http://localhost:9000"; // other host
        let uri = self.create_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut forwarded_req = Request::new(req.method().clone(), uri);
        forwarded_req.headers_mut().extend(req.headers().iter());
        forwarded_req.set_body(req.body());
        forwarded_req
    }
}
