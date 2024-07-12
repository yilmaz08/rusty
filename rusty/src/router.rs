use crate::parser::Route;
use crate::parser::Config;
use crate::parser::Data;
use crate::server;

use libloading::Library;
use std::sync::Arc;

use std::{
    thread,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};
use regex::Regex;

fn read_http_request(stream: TcpStream) -> (Vec<String>, TcpStream) {
    let mut new_stream = stream.try_clone().unwrap();
    let buf_reader = BufReader::new(&mut new_stream);
    // let http_request: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();
    let mut http_request: Vec<String> = Vec::new();
    for line in buf_reader.lines().take_while(|line| !line.as_ref().unwrap_or(&"".to_string()).is_empty()) {
        match line {
            Ok(l) => http_request.push(l),
            Err(_) => println!("Error reading line!"),
        }
    }

    return (http_request, new_stream);
}

fn incoming_request(stream: TcpStream, routers: HashMap<String, Route>) -> (String, HashMap<String, String>, TcpStream) {
    // Stream & Request
    let (http_request, new_stream) = read_http_request(stream);
    
    if http_request.len() == 0 { return ("400".to_string(), HashMap::<String,String>::new(), new_stream); } // Empty or wrong request! (also solves non handled https requests crashing the server)
    
    // Service Data
    let mut data: HashMap<String,String> = HashMap::<String,String>::new();

    // Parse Request - Method, Path, Protocol, Headers
    let request_line = http_request[0].clone();
    let request_array = request_line.split_whitespace().collect::<Vec<&str>>();

    let request_method = request_array[0];
    let request_path = request_array[1];
    let request_protocol = request_array[2];

    data.insert("request_method".to_string(), request_method.to_string());
    data.insert("request_protocol".to_string(), request_protocol.to_string());
    data.insert("request_uri".to_string(), request_path.to_string());

    let mut request_headers: HashMap<String,String> = HashMap::<String,String>::new();
    for line in http_request {
        let split_line: Vec<&str> = line.split(": ").collect();
        if split_line.len() < 2 { continue; } // Skip lines that don't have a colon
        request_headers.insert(split_line[0].to_string(), split_line[1].to_string());
    }

    println!("--- Request ---");
    println!("Method: {request_method}");
    println!("Path: {request_path}");
    println!("Protocol: {request_protocol}");
    println!("Headers: {request_headers:#?}");
    println!("--- Debug ---");

    if !request_headers.contains_key("Host") { return ("400".to_string(), data, new_stream); } // Host header is missing!
    
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
    //

    
    let re = Regex::new(&location_chosen).unwrap();
    let request_uri_regex = re.replace(request_path, "").to_string();
    data.insert("request_uri_non_regex".to_string(), request_uri_regex.to_string());
    data.insert("request_uri_regex".to_string(), request_path.replace(&request_uri_regex, "").to_string());
    
    if size == -1 { return ("404".to_string(), data, new_stream); } // Router & Location is not found!

    return (
        routers.get(&route_chosen).unwrap().paths.get(&location_chosen).unwrap().service.clone(),
        data,
        new_stream
    );
}

fn listen(port: String, config: Config, data: Data, modules: HashMap<String, Arc<Library>>) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("Listening on port {port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let (service, service_data, mut new_stream) = incoming_request(stream, config.http.routes.clone());
                println!("Service: {service} - Port: {port}");
                println!("Data: {service_data:#?}");
                let response = server::respond(service, service_data, data.clone(), config.clone(), modules.clone());
                new_stream.write_all(&response).unwrap();
            }
            Err(e) => { eprintln!("Failed: {e}"); }
        }
    }
    Ok(())
}

pub fn start(config: Config, data: Data, modules: HashMap<String, Arc<Library>>) {
    let mut ports = Vec::<String>::new();

    for route in config.http.routes.keys() {
        for port in config.http.routes.get(route).unwrap().ports.clone() {
            if !ports.contains(&port) {
                println!("Found port {port} in the config");
                ports.push(port);
            }
        }
    }

    for port in ports {
        let (config_copy, data_copy, modules_copy) = (config.clone(), data.clone(), modules.clone());
        thread::spawn(move || {
            let _ = listen(port.clone(), config_copy, data_copy, modules_copy);
        });
    }
}