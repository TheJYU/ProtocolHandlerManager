use crate::config::Config;
use std::process::{Command, exit};
use std::path::Path;
use urlencoding::decode;
use shell_words;
use shellexpand;

pub fn handle_url(url: &str) {
    let decoded_url = decode(url).unwrap_or(std::borrow::Cow::Borrowed(url));
    
    let (protocol, parameters) = match decoded_url.find(':') {
        Some(i) => {
            let p = &decoded_url[..i];
            let mut p_args = &decoded_url[i+1..];
            if p_args.starts_with("//") {
                p_args = &p_args[2..];
            }
            // Strip trailing slash that browsers often add
            if p_args.ends_with('/') {
                p_args = &p_args[..p_args.len() - 1];
            }
            (p, p_args)
        }
        None => {
            eprintln!("Invalid URL format: {}", url);
            exit(1);
        }
    };

    let config = Config::load();
    let raw_target = match config.mappings.get(protocol) {
        Some(t) => t,
        None => {
            eprintln!("No mapping found for protocol: {}", protocol);
            exit(1);
        }
    };

    // 1. Expand environment variables in the Target path
    let expanded_target = shellexpand::full(raw_target)
        .unwrap_or(std::borrow::Cow::Borrowed(raw_target))
        .to_string();

    // 2. Expand environment variables in the Parameters string
    let expanded_parameters = shellexpand::full(parameters)
        .unwrap_or(std::borrow::Cow::Borrowed(parameters))
        .to_string();

    // 3. Parse the parameters string into shell arguments safely
    let args = shell_words::split(&expanded_parameters).unwrap_or_else(|e| {
        eprintln!("Failed to parse arguments: {}", e);
        expanded_parameters.split_whitespace().map(|s| s.to_string()).collect()
    });

    println!("Launching {} with arguments: {:?}", expanded_target, args);

    // 4. Determine the working directory (parent of the target exe)
    let working_dir = Path::new(&expanded_target).parent();

    let mut command = Command::new(&expanded_target);
    command.args(args);
    
    if let Some(dir) = working_dir {
        if dir.exists() {
            command.current_dir(dir);
        }
    }

    let status = command.status();

    if let Err(e) = status {
        eprintln!("Failed to launch target application: {}", e);
        exit(1);
    }
}
