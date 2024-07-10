mod parser;
mod router;

fn main() {
    let config = parser::parse_toml("config.toml".to_string());
    println!("Config is loaded from config.toml");
    // println!("{:#?}", config);
    let _ = router::start(config);
}
