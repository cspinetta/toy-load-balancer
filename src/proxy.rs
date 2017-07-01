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

pub struct Proxy {
    pub client: Client<HttpConnector, Body>
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

        let host = "http://localhost:9000"; // other host
        let uri = self.create_proxy_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut client_req = Request::new(req.method().clone(), uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        info!("Dispatching request: {:?}", client_req);

        let resp = self.client.request(client_req).then(move |result| {
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
