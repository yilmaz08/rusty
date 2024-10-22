use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

pub fn execute_module(lib: &Library, config: HashMap<String, Value>, data: HashMap<String, String>) -> (String, HashMap<String, String>, Vec<u8>, String) {
    unsafe{
        // Answer -> (String, HashMap<String, String>, String) -> StatusCode, Headers, Body
        let func: Symbol<unsafe extern "C" fn(HashMap<String, String>, HashMap<String, Value>) -> (String, HashMap<String, String>, Vec<u8>, String)> = lib.get(b"execute").expect("Failed to load function");
        return func(data, config);
    }
}

fn load_module(module_path: &str) -> Result<Arc<Library>, String> {
    unsafe {
        let lib = Library::new(module_path).unwrap();
        Ok(Arc::new(lib))
    }
}

pub fn load_modules(modules: HashMap<String, String>) -> HashMap<String, Arc<Library>> {
    let mut new_modules: HashMap<String, Arc<Library>> = HashMap::<String, Arc<Library>>::new();

    for (module_name, module_path) in modules {
        match load_module(&module_path) {
            Ok(lib) => { new_modules.insert(module_name.clone(), lib.into()); println!("Module {module_name} successfully loaded from {module_path}!") }
            Err(e) => println!("Failed to load library: {}", e),
        }
    }
    
    return new_modules;
}