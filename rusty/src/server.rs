use crate::parser::Data;
use crate::parser::Config;

use std::collections::HashMap;
use std::io::Read;
use std::fs::File;

pub fn plain_status_code(status_code: String, status_text: String) -> String { // Built-in
    let mut file = File::open("../html/status_code.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    contents = contents.replace("$status_code$", &status_code);
    contents = contents.replace("$status_text$", &status_text);

    return contents;
}

pub fn respond(service_name: String, service_data: HashMap<String, String>, data: Data, config: Config) -> Vec<u8> {
    if !service_data.contains_key("request_protocol") { return "".to_string().into_bytes(); } // Possibly unsupported request type

    let protocol = service_data["request_protocol"].clone();
    let content;
    let mut headers = HashMap::<String,String>::new();
    let mut status_code = "404".to_string(); // Default status code
    let mut status_text = data.status_codes[&status_code].clone(); // Default status text
    
    headers.insert("Server".to_string(), "rusty".to_string());
    headers.insert("Date".to_string(), chrono::Utc::now().to_rfc2822().to_string());
    headers.insert("Content-Type".to_string(), "text/html".to_string());

    if !config.http.services.contains_key(&service_name) { // Service not defined (might be status code)
        if data.status_codes.contains_key(&service_name) {
            status_code = service_name;
            status_text = data.status_codes[&status_code].clone();
        }
        content = plain_status_code(status_code.clone(), status_text.clone());
    }
    else {
        status_code = "501".to_string(); // Services are not implemented! yet :)
        status_text = data.status_codes[&status_code].clone();
        content = plain_status_code(status_code.clone(), status_text.clone());
    }

    headers.insert("Content-Length".to_string(), content.len().to_string());
    // Build Response - Data is ready
    let mut response = format!("{protocol} {status_code} {status_text}\r\n").to_string();   // Add Status Line
    for (key, value) in headers { response.push_str(&format!("{key}: {value}\r\n")); }      // Add Headers
    response.push_str(&format!("\r\n{content}").to_string());                               // Add Content

    println!("--- Response ---\n{response}"); // For debugging
    return response.into_bytes();
}