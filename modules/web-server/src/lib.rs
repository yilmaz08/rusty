use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::fs;

#[no_mangle] // Answer -> Status Code, Additional Headers, buffer, Redirected Service
pub extern "C" fn execute(data: HashMap<String, String>, config: HashMap<String, String>) -> (String, HashMap<String, String>, Vec<u8>, String) {
    let mut buffer = Vec::<u8>::new();
    let mut headers = HashMap::new();
    headers.insert("Rusty-Module".to_string(), "web-server".to_string());

    // Return status code
    if config.contains_key("return") {
        if config.contains_key("location") {
            let mut new_location = config.get("location").unwrap().to_string();
            for key in data.keys() {
                let replacement = data.get(key).unwrap();
                let _key = &format!("${}$", key);
                new_location = new_location.replace(_key, replacement).clone();
            }
            headers.insert("Location".to_string(), new_location);
        }
        return (config.get("return").unwrap().to_string(), headers, buffer, "".to_string());
    }
    // Return file
    if config.contains_key("file") && config.contains_key("root") {
        // Relative paths
        let mut rel_file_path = config.get("file").unwrap().to_string();
        let mut rel_root_path = config.get("root").unwrap().to_string();
        
        // Replace variables in paths
        for key in data.keys() {
            let replacement = data.get(key).unwrap();
            let _key = &format!("${}$", key);
            rel_file_path = rel_file_path.replace(_key, replacement).clone();
            rel_root_path = rel_root_path.replace(_key, replacement).clone();
        }

        // Root validity
        if !std::path::Path::new(&rel_root_path).exists() { return ("403".to_string(), headers, buffer, "403".to_string()); }
        if !std::fs::metadata(&rel_root_path).unwrap().is_dir() { return ("403".to_string(), headers, buffer, "403".to_string()); }

        // Absolute root path
        let abs_root_path = match fs::canonicalize(rel_root_path) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error converting root to absolute: {}", e);
                return ("404".to_string(), headers, buffer, "404".to_string());
            }
        };
    
        // File in root
        let comb_file_path = abs_root_path.join(rel_file_path);

        // File validity
        if !std::path::Path::new(&comb_file_path).exists() { return ("404".to_string(), headers, buffer, "404".to_string()); }
        if std::fs::metadata(&comb_file_path).unwrap().is_dir() { return ("404".to_string(), headers, buffer, "404".to_string()); }

        // Absolute file path
        let abs_file_path = match fs::canonicalize(comb_file_path) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error converting file to absolute: {}", e);
                return ("404".to_string(), headers, buffer, "404".to_string());
            }
        };

        // Check if file is outside root
        if !abs_file_path.starts_with(abs_root_path) {
            // Danger
            return ("403".to_string(), headers, buffer, "403".to_string());
        }
        
        // Read file
        let mut file = File::open(&abs_file_path).unwrap();
        file.read_to_end(&mut buffer).unwrap();

        // Return
        return ("200".to_string(), headers, buffer, "".to_string());
    }
    // Return 404
    return ("404".to_string(), headers, buffer, "404".to_string());
}