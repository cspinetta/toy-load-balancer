use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::Result;


// INPUT FILE -> property=value, e.g. host1=http://localhost:9000
// OUTPUT [(property1,value1),...,(propertyN,valueN)]
pub fn read_file() -> Result<Vec<(String,String)>> {
    let mut properties = Vec::new();
	let file = File::open("config-file.txt")?; 
	let reader = BufReader::new(file);
    for line in reader.lines() {
        let mut property = String::from(line.unwrap());
        let token_index = property.find('=').unwrap()+1;
        let value = property.split_off(token_index);
        properties.push((property,value));
    }
    Ok(properties)
}


//let file_properties_res = file_utils::read_file();
//let file_properties = file_properties_res.unwrap();
//let host = &file_properties.get(0).unwrap().1; 
        