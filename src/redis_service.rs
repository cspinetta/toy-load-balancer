extern crate redis;

use self::redis::Commands;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use self::redis::{RedisError, ErrorKind, RedisResult, ToRedisArgs};


pub trait Cache {

    fn set(&self, key: &str, value: String) -> redis::RedisResult<()>;

    fn get(&self, key: &str) -> redis::RedisResult<String>;
}

pub struct ImplCache {
	con: Arc<Mutex<RedisResult<redis::Connection>>>,
    redis_conn: Arc<String>
}

impl ImplCache {

    pub fn new(redis_conn: String) -> ImplCache {
        ImplCache { con: Arc::new(Mutex::new(Self::create_connection(redis_conn.clone()))), redis_conn: Arc::new(redis_conn.clone()) }
    }

    fn create_connection(redis_conn: String) -> RedisResult<redis::Connection> {
        let client = redis::Client::open(redis_conn.clone().as_str()).expect("Create connection to Redis");
        let conn = client.get_connection();
        conn.as_ref().map_err(|e| {
            error!("Failed trying to connect to Redis on {}", redis_conn);
            e
        });
        conn
    }

}

impl Cache for ImplCache {

    fn set(&self, key: &str, value: String) -> redis::RedisResult<()> {
        info!("Enter to save {} in Cache...", key);
        let lock = self.con.lock();
        match lock {
            Ok(conn_result) => {
                match conn_result.as_ref() {
                    Ok(c) => {
                        info!("Saving {} in Cache...", key);
                        c.set(key.clone(), value.clone())
                            .map_err(|error| {
                                error!("Failed trying to save {} on Redis", key.clone());
                                error
                            })
                    },
                    Err(e) => Err(RedisError::from((ErrorKind::NoScriptError, "No connection acquired")))
                }
            },
            Err(e) => Err(RedisError::from((ErrorKind::NoScriptError, "No lock for get cache connection")))
        }
    }

    fn get(&self, key: &str) -> redis::RedisResult<String> {
        info!("Enter to get {} in Cache...", key);
        match self.con.lock() {
            Ok(conn_result) =>
                match conn_result.as_ref() {
                    Ok(c) => {
                        info!("Saving {} in Cache...", key);
                        c.get(key.clone())
                            .map_err(|error| {
                                error!("Failed trying to fetch {} from Redis", key.clone());
                                error
                            })
                    },
                    Err(e) => Err(RedisError::from((ErrorKind::NoScriptError, "No connection acquired")))
                },
            Err(e) => Err(RedisError::from((ErrorKind::NoScriptError, "No lock for get cache connection")))
        }
    }
}

pub struct NoOpCache;

impl NoOpCache {
    pub fn new() -> NoOpCache {
        NoOpCache {}
    }
}

impl Cache for NoOpCache {
    fn set(&self, key: &str, value: String) -> redis::RedisResult<()> {
        info!("NoOp Cache in save operation for key {}", key);
        Ok(())
    }

    fn get(&self, key: &str) -> redis::RedisResult<String> {
        info!("NoOp Cache in save operation for key {}", key);
        Err(RedisError::from((ErrorKind::NoScriptError, "No Op Cache")))
    }
}
