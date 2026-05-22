use std::process::{self, Command};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("compile") => compile(),
        Some("lint") => lint(),
        Some("test") => test(),
        Some("package") => package(),
        _ => {
            eprintln!("usage: cargo xtask <subcommand>");
            eprintln!("subcommands: compile, lint, test, package");
            process::exit(1);
        }
    }
}

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to execute '{cmd}': {e}"));
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

fn npm() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn compile() {
    run(npm(), &["run", "compile"]);
}

fn lint() {
    run("cargo", &["clippy", "--", "-D", "warnings"]);
    run("cargo", &["fmt", "--check"]);
    run(npm(), &["run", "lint"]);
}

fn test() {
    run("cargo", &["test"]);
    run("cargo", &["test", "--target", "wasm32-unknown-unknown"]);
}

fn package() {
    run(npm(), &["run", "package"]);
}
