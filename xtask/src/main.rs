//! 开发工具：`cargo xtask layers` 检查分层，`cargo xtask deps` 检查依赖预算。

use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, ExitCode};

use serde_json::Value;

/// 分层表（docs: book/src/architecture/crates.md）。新 crate 必须登记在这里。
const LAYERS: &[(&str, u8)] = &[
    ("aa4c-types", 0),
    ("aa4c-wire", 0),
    ("aa4c-id", 1),
    ("aa4c-state", 1),
    ("aa4c-net", 2),
    ("aa4c-files", 3),
    ("aa4c-plugins", 3),
    ("aa4c-engine", 4),
    ("aa4c", 5),
    ("aa4c-desktop", 5),
    ("aa4c-mobile", 5),
];

/// 不属于产品分层的开发工具。
const TOOLS: &[&str] = &["xtask"];

/// 界面相关的库只许出现在 L5。
const L5_ONLY_DEPS: &[&str] = &["egui", "eframe", "slint", "uniffi", "rmcp"];

/// `aa4c` 无界面版的运行时依赖上限（去重后的 crate 数）。
const DEPS_BUDGET: usize = 130;

type Result<T> = std::result::Result<T, String>;

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("layers") => layers(),
        Some("deps") => deps(),
        _ => Err("usage: cargo xtask <layers|deps>".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {} failed:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("{program} output is not UTF-8: {e}"))
}

fn layers() -> Result<()> {
    let metadata = run("cargo", &["metadata", "--format-version", "1", "--no-deps"])?;
    let metadata: Value = serde_json::from_str(&metadata).map_err(|e| e.to_string())?;
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("cargo metadata has no packages")?;

    let table: BTreeMap<&str, u8> = LAYERS.iter().copied().collect();
    let mut errors = Vec::new();
    let mut members = BTreeSet::new();

    for package in packages {
        let name = package.get("name").and_then(Value::as_str).unwrap_or("");
        members.insert(name.to_string());
        if TOOLS.contains(&name) {
            continue;
        }
        let Some(&layer) = table.get(name) else {
            errors.push(format!("{name}: not registered in the layer table"));
            continue;
        };
        let deps = package
            .get("dependencies")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for dep in deps {
            let dep_name = dep.get("name").and_then(Value::as_str).unwrap_or("");
            if let Some(&dep_layer) = table.get(dep_name)
                && dep_layer > layer
            {
                errors.push(format!(
                    "{name} (L{layer}) depends on {dep_name} (L{dep_layer}): lower layers must not depend on higher ones"
                ));
            }
            if layer < 5 && L5_ONLY_DEPS.contains(&dep_name) {
                errors.push(format!(
                    "{name} (L{layer}) depends on {dep_name}, which is allowed only in L5"
                ));
            }
        }
    }
    for (name, _) in LAYERS {
        if !members.contains(*name) {
            errors.push(format!(
                "{name}: in the layer table but not in the workspace"
            ));
        }
    }

    if errors.is_empty() {
        println!("layers: ok ({} crates)", LAYERS.len());
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn deps() -> Result<()> {
    let tree = run(
        "cargo",
        &[
            "tree", "-p", "aa4c", "-e", "normal", "--prefix", "none", "--format", "{p}",
        ],
    )?;
    // Workspace members have a path source, shown as "(/path)"; external crates do not.
    let external: BTreeSet<&str> = tree
        .lines()
        .map(|line| line.trim_end_matches(" (*)"))
        .filter(|line| !line.is_empty() && !line.contains(" ("))
        .collect();
    for entry in &external {
        println!("{entry}");
    }
    println!(
        "deps: {} unique external crates (budget {DEPS_BUDGET})",
        external.len()
    );
    if external.len() > DEPS_BUDGET {
        return Err(format!("over budget: {} > {DEPS_BUDGET}", external.len()));
    }
    Ok(())
}
