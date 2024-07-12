use crate::parser::Route;
use crate::parser::Config;
use crate::parser::Data;
use crate::server;

use std::{
    thread,
    str,
    io::{prelude::*},
    net::{TcpListener, TcpStream},
    collections::HashMap,
    collections::HashSet,
};
use regex::Regex;
use libloading::Library;
use std::sync::Arc;


// Return -> stream, request_data, peer_ip
fn read_http_request(mut stream: TcpStream, mut max_body_size: i32) -> (TcpStream, String, Vec<String>) {
    let peer_ip = stream.peer_addr().unwrap().ip();
    let mut request_str: String = String::new();

    loop {
        let mut buf = [0; 512];
        let n = stream.read(&mut buf[..]).unwrap();
        
        max_body_size -= n as i32;
        if 0 > max_body_size { return (stream, peer_ip.to_string(), Vec::<String>::new()); }
        
        request_str.push_str(&std::str::from_utf8(&buf[..n]).unwrap().to_string());

        if buf.contains(&0) { break; } // end
    }
    let request = request_str.split("\r\n").map(|x| x.to_string()).collect::<Vec<String>>();

    println!("{:#?}", request);
    
    return (stream, peer_ip.to_string(), request); // Debugging - Stop here
}

// Return -> service, service_data, stream
fn incoming_request(stream: TcpStream, routers: HashMap<String, Route>, max_body_size: i32) -> (String, HashMap<String, String>, TcpStream) {
    // Service Data
    let mut service_data: HashMap<String,String> = HashMap::<String,String>::new();

    // Stream & Request
    let (new_stream, peer_ip, http_request) = read_http_request(stream, max_body_size); // 50MB max body size
    
    service_data.insert("peer_ip".to_string(), peer_ip);

    if http_request.len() == 0 { return ("400".to_string(), HashMap::<String,String>::new(), new_stream); } // Empty or wrong request! (also solves non handled https requests crashing the server)
    
    // Parse Request - Method, Path, Protocol, Headers
    let request_line = http_request[0].clone();
    let request_array = request_line.split_whitespace().collect::<Vec<&str>>();

    let request_method = request_array[0];
    let request_path = request_array[1];
    let request_protocol = request_array[2];

    service_data.insert("request_method".to_string(), request_method.to_string());
    service_data.insert("request_uri".to_string(), request_path.to_string());
    service_data.insert("request_protocol".to_string(), request_protocol.to_string());

    let mut request_headers: HashMap<String,String> = HashMap::<String,String>::new();
    let mut request_body: Vec<String> = Vec::<String>::new();

    let mut on_body = false;

    for line in http_request {
        if !on_body {
            if line == "" { on_body = true; }
            
            let split_line: Vec<&str> = line.split(": ").collect();
            if split_line.len() < 2 { continue; }
            request_headers.insert(split_line[0].to_string(), split_line[1].to_string());
        }
        else { request_body.push(line); }
    }

    println!("--- Request ---");
    println!("Method: {}", request_method);
    println!("Path: {}", request_path);
    println!("Protocol: {}", request_protocol);
    println!("Headers: {request_headers:#?}");
    println!("Body: {request_body:#?}");
    println!("--- Debug ---");

    if !request_headers.contains_key("Host") { return ("400".to_string(), service_data, new_stream); } // Host header is missing!
    
    // Host & Port
    let split_host = request_headers.get("Host").unwrap().split(":").collect::<Vec<&str>>();
    let host = split_host[0].trim();
    let port = split_host[1];

    println!("Host: \"{host}\"");
    println!("Port: \"{port}\"");

    // Find all routes
    let route_keys: Vec<_> = routers.keys().cloned().collect();
    println!("Route keys: {route_keys:#?}");

    let mut matching: Vec<String> = Vec::<String>::new(); // Host abd port matching

    // Match by port and hosts here
    for key in route_keys {
        if routers.get(&key).unwrap().ports.contains(&port.to_string()) && routers.get(&key).unwrap().hosts.contains(&host.to_string()) {
            matching.push(key);
        }
    }
    println!("Matching: {matching:#?}");
    
    let mut route_chosen: String = "".to_string();
    let mut location_chosen: String = "".to_string();
    let mut size: i128 = -1;

    // Needs fixing: TODO
    for router in matching {
        let paths = routers.get(&router).unwrap().paths.keys().cloned().collect::<Vec<String>>();
        println!("Paths: {:#?}", paths);

        for path in paths {
            let re = Regex::new(&path).unwrap();
            let is_match = re.is_match(request_path);
            println!("Is match: {is_match} {path}");
            if is_match {
                let path_left = re.replace(request_path, "");
                if size != -1 && location_chosen == path { // same location, different router - look at priority
                    let current_priority = routers.get(&route_chosen).unwrap().priority;
                    let new_priority = routers.get(&router).unwrap().priority;
                    if new_priority > current_priority {
                        route_chosen = router.clone();
                        location_chosen = path.clone();
                        size = path_left.len().try_into().unwrap();
                    }
                }
                else if size == -1 || size > path_left.len().try_into().unwrap() {
                    route_chosen = router.clone();
                    location_chosen = path.clone();
                    size = path_left.len().try_into().unwrap();
                }
            }
        }
    }
    
    let re = Regex::new(&location_chosen).unwrap();
    let request_uri_regex = re.replace(request_path, "").to_string();
    service_data.insert("request_uri_non_regex".to_string(), request_uri_regex.to_string());
    service_data.insert("request_uri_regex".to_string(), request_path.replace(&request_uri_regex, "").to_string());
    service_data.insert("request_uri_trimmed".to_string(), request_path.trim_start_matches('/').trim_end_matches('/').to_string());
    
    if size == -1 { return ("404".to_string(), service_data, new_stream); } // Router & Location is not found!

    return (
        routers.get(&route_chosen).unwrap().paths.get(&location_chosen).unwrap().to_string(),
        service_data,
        new_stream
    );
}

fn handle_request(stream: TcpStream, config: Config, data: Data, modules: HashMap<String, Arc<Library>>) {
    let (service, service_data, mut new_stream) = incoming_request(stream, config.http.routes.clone(), config.http.max_body_size.try_into().unwrap());
    println!("Service: {service}");
    println!("Data: {service_data:#?}");
    new_stream.write_all(&server::respond(service, service_data, data, config, modules)).unwrap();
}

fn listen(port: String, config: Config, data: Data, modules: HashMap<String, Arc<Library>>) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("Listening on port {port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config_clone = config.clone();
                let data_clone = data.clone();
                let modules_clone = modules.clone();

                // Single-Thread
                // handle_request(stream, config_clone, data_clone, modules_clone);
                
                // Multi-Thread
                thread::spawn(move || {
                    handle_request(stream, config_clone, data_clone, modules_clone);
                });
            }
            Err(e) => { eprintln!("Failed: {e}"); }
        }
    }
    Ok(())
}

pub fn start(config: Config, data: Data, modules: HashMap<String, Arc<Library>>) {
    let mut ports = HashSet::<String>::new();
    
    for route in config.http.routes.keys() {
        ports.extend(config.http.routes.get(route).unwrap().ports.clone());
    }

    for port in ports {
        let (config_copy, data_copy, modules_copy) = (config.clone(), data.clone(), modules.clone());
        thread::spawn(move || {
            let _ = listen(port.clone(), config_copy, data_copy, modules_copy);
        });
    }
}