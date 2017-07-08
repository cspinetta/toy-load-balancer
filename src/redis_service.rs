extern crate redis;

use self::redis::Commands;
use std::sync::Arc;
use self::redis::{RedisError, ErrorKind};


pub trait Cache {

    fn set(&self, key: String, value: String) -> redis::RedisResult<()>;

    fn get(&self, key: String) -> redis::RedisResult<String>;
}

pub struct ImplCache {
	pub con: Arc<redis::Connection>,
    redis_conn: String
}

impl ImplCache {

    pub fn new(redis_conn: String) -> ImplCache {
        ImplCache { con: Arc::new(Self::create_connection(redis_conn.clone())), redis_conn: redis_conn.clone() }
    }

    fn create_connection(redis_conn: String) -> redis::Connection {
        let client = redis::Client::open(redis_conn.clone().as_str()).expect("Create connection to Redis");
        return client.get_connection().unwrap();
    }

}

impl Cache for ImplCache {

    fn set(&self, key: String, value: String) -> redis::RedisResult<()> {
        info!("Saving {} in Cache...", key);
        let _ : () = try!(self.con.clone().set(key, value));
        Ok(())
    }

    fn get(&self, key: String) -> redis::RedisResult<String> {
        let value: String  = try!(self.con.clone().get(key));
        Ok(value)
    }
}

pub struct NoOpCache;

impl NoOpCache {
    pub fn new() -> NoOpCache {
        NoOpCache {}
    }
}

impl Cache for NoOpCache {
    fn set(&self, key: String, value: String) -> redis::RedisResult<()> {
        info!("NoOp Cache in save operation for key {}", key);
        Ok(())
    }

    fn get(&self, key: String) -> redis::RedisResult<String> {
        info!("NoOp Cache in save operation for key {}", key);
        Err(RedisError::from((ErrorKind::NoScriptError, "No Op Cache")))
    }
}
