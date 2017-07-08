extern crate redis;

pub struct Cache{
	pub con : redis::Connection
}

pub fn create_connection() -> redis::Connection {
    let client = try!(redis::Client::open("redis://127.0.0.1:6379/"));
    return try!(client.get_connection());
}

pub fn set(con: redis::Connection,key: String,value: String) -> redis::RedisResult<String> {
    let _ : () = try!(con.set(key, value));
    Ok(())
}

pub fn get(con: redis::Connection,key: String) -> redis::RedisResult<()> {
    let value : String  = try!(con.get(key));
    Ok((value))
}