use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::Result;

pub struct FileReader{
}

impl FileReader{
	// INPUT  -> FILE, property=value, e.g. host1=http://localhost:3001
	// OUTPUT -> [(property1,value1),...,(propertyN,valueN)]
	pub fn read() -> Result<Vec<String>> {
	    let mut properties = Vec::new();
		let file = File::open("config-file.txt")?; 
		let reader = BufReader::new(file);
	    for line in reader.lines() {
	        let mut property = String::from(line.unwrap());
	        let token_index = property.find('=').unwrap()+1;
	        let value = property.split_off(token_index);
	        properties.push(value);
	    }
	    Ok(properties)
	}	
}
