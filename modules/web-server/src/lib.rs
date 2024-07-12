use std::collections::HashMap;
use std::io::Read;
use std::fs::File;


// TEMPORARY //

#[no_mangle] // Answer -> Status Code, Additional Headers, Body
pub extern "C" fn execute(data: HashMap<String, String>, config: HashMap<String, String>) -> (String, HashMap<String, String>, String) {
    let mut headers = HashMap::new();
    headers.insert("Module".to_string(), "web-server".to_string());

    let mut body = String::new();
    if config.contains_key("index") {
        let file_path = config.get("index").unwrap();
        let mut file = File::open(&file_path).unwrap();
        file.read_to_string(&mut body).unwrap();
    }

    return ("200".to_string(), headers, body);
}
