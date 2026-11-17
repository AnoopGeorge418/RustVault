use std::env;
use std::process;

fn main() {
    intro_banner();

    let vault_command: Vec<String> = env::args().collect();
    vault_commands(&vault_command);
}

fn tasks() {
    println!("Add new tasks with supported options");
}

fn secrets() {
    println!("Add new secret with supported options");
}

fn projects() {
    println!("Add new project with supported options");
}

fn dashboard() {
    println!("View dashboard with supported options");
}

fn settings() {
    println!("Settings with supported options");
}

fn exit_app() {
    println!("Leaving Already 👋, All your data are safe with us! Come back any time...");
    process::exit(1);
}

/// List of commands
///
/// Short info on how to use it
/// - cargo run 'command'
fn help() {
    println!("Available Commands: ");
    println!("Tasks");
    println!("Secrets Vault");
    println!("Projects");
    println!("Dashboard");
    println!("Settings");
    println!("Exit");
    println!("Help");

    println!("");
    println!("Please enter these commands as it is: ");
    println!("Example: cargo run tasks");
}

fn vault_commands(args: &[String]) {
    if args.len() <= 1 {
        panic!("Please enter the command");
    }

    let args = args[1].to_lowercase();

    if args == "tasks" {
        tasks();
    } else if args == "secrets" {
        secrets();
    } else if args == "projects" {
        projects();
    } else if args == "dashboard" {
        dashboard();
    } else if args == "settings" {
        settings();
    } else if args == "exit" {
        exit_app()
    } else if args == "help" {
        help();
    } else {
        panic!("Command not found!");
    }
}

/// Prints the RustVault crate banner.
///
/// Displays the crate name and its description
/// - inside a decorative terminal box.
fn intro_banner() {
    println!("╭───────────────────────────────────────────────╮");
    println!("│               🦀 RUST VAULT                   │");
    println!("│         Your personal command center          │");
    println!("╰───────────────────────────────────────────────╯");
}
