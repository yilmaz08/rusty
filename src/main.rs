mod parser;
mod router;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(help = "Action to perform: start or test")]
    action: String,

    #[arg(short, long, default_value = "./config.toml")]
    config: String,
}

fn main() {
    let args = Args::parse();

    match args.action.as_str() {
        "start" => start(args.config),
        "test" => test(args.config),
        _ => println!("Unknown action: {}", args.action),
    }
}

fn test(config_path: String) {
    let _ = parser::parse_config(config_path.to_string());
    println!("Config was loaded from file without any errors: {}", config_path);
}

fn start(config_path: String) {
    let config = parser::parse_config(config_path.to_string());
    println!("Config is loaded from file: {}", config_path);
    // println!("{:#?}", config);
    router::start(config);
    std::thread::park();
}