mod parser;
mod router;
mod server;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(help = "Action to perform: start or test")]
    action: String,

    #[arg(short, long, default_value = "../config/rusty.toml")]
    config: String,

    #[arg(short, long, default_value = "../config/data.toml")]
    data: String,
}

fn main() {
    let args = Args::parse();

    match args.action.as_str() {
        "start" => start(args.config, args.data),
        "test" => test(args.config, args.data),
        _ => println!("Unknown action: {}", args.action),
    }
}

fn test(config_path: String, data_path: String) {
    let _config = parser::parse_config(config_path.to_string());
    println!("Config was loaded from file without any errors: {}", config_path);
    let _data = parser::parse_data(data_path.to_string());
    println!("Data was loaded from file without any errors: {}", data_path);
}

fn start(config_path: String, data_path: String) {
    let config = parser::parse_config(config_path.to_string());
    println!("Config is loaded from file: {}", config_path);

    let data = parser::parse_data(data_path.to_string());
    println!("Data is loaded from file: {}", data_path.to_string());

    // println!("{:#?}", config);
    router::start(config, data);
    std::thread::park();
}