//! AA4C 无界面版：常驻、CLI、本地 IPC、MCP、中转站。

use std::process::ExitCode;

use aa4c_engine::Engine;

fn commands(json: bool) -> Result<(), String> {
    let engine = Engine::new();
    let descriptors = engine.descriptors();
    if json {
        let text = serde_json::to_string_pretty(&descriptors).map_err(|e| e.to_string())?;
        println!("{text}");
    } else {
        for d in &descriptors {
            let tier = serde_json::to_value(d.tier)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            println!("{:<24}{:<7}{}", d.id, tier, d.summary);
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["commands"] => commands(false),
        ["commands", "--json"] => commands(true),
        _ => {
            eprintln!("usage: aa4c commands [--json]");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
