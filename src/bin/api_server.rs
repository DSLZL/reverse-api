mod api;

use reverse_api::Logger;
use std::env;

fn print_usage() {
    println!("Usage: api_server [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --host <HOST>      Server host (default: 0.0.0.0)");
    println!("  --port <PORT>      Server port (default: 6969)");
    println!("  --proxy <PROXY>    Default proxy for Grok client (default: http://127.0.0.1:1082)");
    println!("  --no-proxy         Don't use any default proxy");
    println!("  --help             Show this help message");
    println!();
    println!("Note: Proxy can also be overridden per-request in the API payload.");
    println!();
    println!("Examples:");
    println!("  api_server");
    println!("  api_server --port 8080");
    println!("  api_server --proxy http://127.0.0.1:7890");
    println!("  api_server --no-proxy");
    println!("  api_server --host 127.0.0.1 --port 8080 --proxy http://localhost:7890");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut host = "0.0.0.0".to_string();
    let mut port = 6969u16;
    let mut default_proxy = Some("http://127.0.0.1:1082".to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 2;
                } else {
                    Logger::error("--host requires a value");
                    std::process::exit(1);
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(p) => port = p,
                        Err(_) => {
                            Logger::error(&format!("Invalid port number: {}", args[i + 1]));
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    Logger::error("--port requires a value");
                    std::process::exit(1);
                }
            }
            "--proxy" => {
                if i + 1 < args.len() {
                    default_proxy = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    Logger::error("--proxy requires a value");
                    std::process::exit(1);
                }
            }
            "--no-proxy" => {
                default_proxy = None;
                i += 1;
            }
            _ => {
                Logger::error(&format!("Unknown option: {}", args[i]));
                println!();
                print_usage();
                std::process::exit(1);
            }
        }
    }

    if let Ok(env_host) = env::var("API_HOST") {
        host = env_host;
    }

    if let Ok(env_port) = env::var("API_PORT") {
        if let Ok(p) = env_port.parse::<u16>() {
            port = p;
        }
    }

    if let Ok(env_proxy) = env::var("DEFAULT_PROXY") {
        default_proxy = Some(env_proxy);
    }

    Logger::info("Starting Grok-API Server");
    Logger::info("=======================");
    Logger::info(&format!("Host: {}", host));
    Logger::info(&format!("Port: {}", port));

    if let Some(ref proxy) = default_proxy {
        Logger::info(&format!("Default Proxy: {}", proxy));
    } else {
        Logger::info("Default Proxy: None");
    }
    Logger::info("Supported Models: Grok (grok-*) and ChatGPT (chatgpt, gpt-*)");

    if let Err(err) = api::server::run(&host, port, default_proxy).await {
        Logger::error(&format!("API server failed: {}", err));
        std::process::exit(1);
    }
    Logger::success("API server stopped");
}
