mod parser;

fn main() {
    let config = parser::parse_toml("config.toml".to_string());
    println!("{:#?}", config);
}
