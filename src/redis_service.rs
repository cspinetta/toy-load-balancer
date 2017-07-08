extern crate redis;

use self::redis::Commands;
use std::sync::Arc;

pub struct Cache {
	pub con: Arc<redis::Connection>
}

impl Cache {

    pub fn new() -> Cache {
        Cache { con: Arc::new(Self::create_connection()) }
    }

    pub fn set(&self, key: String, value: String) -> redis::RedisResult<()> {
        info!("Saving {} in Cache...", key);
        let _ : () = try!(self.con.clone().set(key, value));
        Ok(())
    }

    pub fn get(&self, key: String) -> redis::RedisResult<String> {
        let value: String  = try!(self.con.clone().get(key));
        Ok(value)
    }

    fn create_connection() -> redis::Connection {
        let client = redis::Client::open("redis://127.0.0.1:6379/").expect("Connect to Redis");
        return client.get_connection().unwrap();
    }
}
