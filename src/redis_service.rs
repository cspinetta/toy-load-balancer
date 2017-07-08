extern crate redis;

use self::redis::Commands;
use std::sync::Arc;

pub struct Cache{
	pub con : redis::Connection
}

pub fn create_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379/").expect("Connect to Redis");
    return client.get_connection().unwrap();
}

pub fn set(con: Arc<redis::Connection>, key: String, value: String) -> redis::RedisResult<()> {
    let _ : () = try!(con.set(key, value));
    Ok(())
}

pub fn get(con: Arc<redis::Connection>, key: String) -> redis::RedisResult<String> {
    let value : String  = try!(con.get(key));
    Ok(value)
}