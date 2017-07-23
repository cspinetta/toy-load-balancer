
use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response};
use hyper::header::ContentLength;

pub fn get_content_length(response: &Response) -> Option<u64> {
    response.headers().get::<ContentLength>().map(|length| {length.0})
}

pub fn get_content_length_req(req: &Request<Body>) -> Option<u64> {
    req.headers().get::<ContentLength>().map(|length| {length.0})
}
