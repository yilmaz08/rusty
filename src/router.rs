use std::{
    thread,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};
use crate::parser::Route;
use crate::parser::Config;
use regex::Regex;


fn incoming_request(stream: TcpStream, routers: HashMap<String, Route>) -> (String, HashMap<String, String>, TcpStream) {
    // Stream & Request
    let mut new_stream = stream.try_clone().unwrap();
    let buf_reader = BufReader::new(&mut new_stream);
    let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap())
    .take_while(|line| !line.is_empty())
    .collect();
    
    // Service Data
    let mut data: HashMap<String,String> = HashMap::<String,String>::new();

    // Parse Request - Method, Path, Protocol, Headers
    let request_line = http_request[0].clone();
    let request_array = request_line.split_whitespace().collect::<Vec<&str>>();

    let request_method = request_array[0];
    let request_path = request_array[1];
    let request_protocol = request_array[2];

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

    let mut ports_matching: Vec<String> = Vec::<String>::new();
    let mut hosts_matching: Vec<String> = Vec::<String>::new();

    // Match by port here
    for key in route_keys {
        if routers.get(&key).unwrap().ports.contains(&port.to_string()) {
            ports_matching.push(key);
        }
    }
    println!("Ports matching: {ports_matching:#?}");
    // Match by host here
    for key in &ports_matching {
        if routers.get(key).unwrap().hosts.contains(&host.to_string()) {
            hosts_matching.push(key.to_string());
        }
    }
    println!("Hosts matching: {hosts_matching:#?}");
    
    let mut route_chosen: String = "".to_string();
    let mut location_chosen: String = "".to_string();
    let mut size: i128 = -1;

    for router in hosts_matching {
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
    data.insert("request_uri_non_regex".to_string(), request_uri_regex.to_string());
    data.insert("request_uri_regex".to_string(), request_path.replace(&request_uri_regex, "").to_string());
    
    if size == -1 { return ("404".to_string(), data, new_stream); } // Router & Location is not found!

    return (
        routers.get(&route_chosen).unwrap().paths.get(&location_chosen).unwrap().service.clone(),
        data,
        new_stream
    );
}

fn respond(service: String) -> Vec<u8> { // TEMPORARY RESPONSE TO TEST ROUTER
    println!("--- Response ---");
    let mut s = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
    if service == "200" {
        s = "HTTP/1.1 200 OK\r\n\r\n".to_string();
    } else if service == "400" {
        s = "HTTP/1.1 400 Bad Request\r\n\r\n".to_string();
    } else if service == "301" {
        s = "HTTP/1.1 301 Moved Permanently\r\n\r\n".to_string();
    }
    s += format!("Service: {:?}", &service).as_str();
    println!("{s}");
    return s.into_bytes();
}

fn listen(port: String, config: Config) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("Listening on port {port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let (service, data, mut new_stream) = incoming_request(stream, config.http.routes.clone());
                println!("Service: {service} - Port: {port}");
                println!("Data: {data:#?}");
                let response = respond(service);
                new_stream.write_all(&response).unwrap();
            }
            Err(e) => { eprintln!("Failed: {e}"); }
        }
    }
    Ok(())
}

pub fn start(config: Config) {
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
        let config_copy = config.clone();
        thread::spawn(move || {
            let _ = listen(port.clone(), config_copy);
        });
    }

    std::thread::park();
}