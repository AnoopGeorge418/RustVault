use std::env;
use std::io::stdin;
use std::process;

fn main() -> std::io::Result<()> {
    intro_banner();

    let vault_command: Vec<String> = env::args().collect();
    vault_commands(&vault_command)?;

    Ok(())
}

#[derive(Debug)]
enum ApiCommands {
    Create,
    View,
    Update,
    Delete,
}

/// Tasks api for creating, viewing, updating and deleting tasks
///
/// uses same ApiCommands Enum
fn tasks() -> std::io::Result<()> {
    println!("Welcome to RustVault task manager!");
    println!("Available Commands are: ");
    println!("- {:?} :: Create new task", ApiCommands::Create);
    println!("- {:?}   :: View all the tasks", ApiCommands::View);
    println!("- {:?} :: Update specific task", ApiCommands::Update);
    println!("- {:?} :: Delete a task", ApiCommands::Delete);
    println!("");

    let mut user_choice = String::new();
    stdin()
        .read_line(&mut user_choice)?;

    match user_choice.trim().to_lowercase().as_str() {
        "create" => create(),
        "view" => view(),
        "update" => update(),
        "delete" => delete(),
        "" => println!("Please enter a command"),
        _ => println!("Command not found!"),
    }

    Ok(())
}

/// Secrets api for creating, viewing, updating and deleting secrets
/// - Always stored as encrypted - for security
///
/// uses same ApiCommands Enum
fn secrets() {
    println!("Welcome to RustVault secrets!");
    println!("Available Commands are: ");
    println!("{:?}", ApiCommands::Create);
    println!("{:?}", ApiCommands::View);
    println!("{:?}", ApiCommands::Update);
    println!("{:?}", ApiCommands::Delete);
    println!("");

    let mut user_choice = String::new();
    stdin()
        .read_line(&mut user_choice)
        .expect("Failed to read tasks command!");

    match user_choice.trim().to_lowercase().as_str() {
        "create" => create(),
        "view" => view(),
        "update" => update(),
        "delete" => delete(),
        "" => println!("Please enter a command"),
        _ => println!("Command not found!"),
    }
}

#[derive(Default, Debug)]
enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    OnHold,
    Completed,
    Cancelled
}

#[derive(Default, Debug)]
enum TaskPriority {
    #[default]
    Low,
    Medium,
    High,
    Urgent
}

#[derive(Default, Debug)]
struct TaskData {
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    due_date: Option<String>,
    start_date: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
}

/// Creates a new task or secret
fn create() {
    println!("Creating task");

    let task = TaskData::default();
    println!("Title: {}", task.title);
    println!("{task:#?}");
}

fn view() {
    println!("Viewing all task");
}

fn update() {
    println!("updating task");
}

fn delete() {
    println!("deleting task");
}

#[derive(Debug)]
enum _DashboardCommands {
    Statistics,
}

fn dashboard() {
    println!("View dashboard with supported options");
}

#[derive(Debug)]
enum _SettingsCommands {
    Export,
    Import,
    Wipe,
}

fn settings() {
    println!("Settings with supported options");
}

/// exit function - that stops rust vault
fn exit_app() {
    println!("Leaving Already 👋, All your data are safe with us! Come back any time...");
    process::exit(0); // 1 refers to error and exit
}

/// Available commands enum for rust vault
///
/// Only these commands are supported throughout rust vault
#[derive(Debug)]
enum VaultCommands {
    Tasks,
    Secrets,
    Dashboard,
    Settings,
    Exit,
    Help,
}

/// List of commands
///
/// Short info on how to use it
/// - cargo run 'command'
fn help() {
    println!("RustVault is a simple terminal application / rust crate built using rust programming language to keep track of tasks, 
save credentials as secret with encryption and remove whenever u want. 
Everything will saved on your system.");

    println!("");

    println!("Available Commands: ");
    println!("{:?} - Create new task with metadata", VaultCommands::Tasks);
    println!(
        "{:?} - Create secrets and encrypt it",
        VaultCommands::Secrets
    );
    println!(
        "{:?} - View statistics and analysis",
        VaultCommands::Dashboard
    );
    println!("{:?} - Import and export data", VaultCommands::Settings);
    println!("{:?} - Exit from rust vault", VaultCommands::Exit);
    println!(
        "{:?} - Get detailed info about rust vault",
        VaultCommands::Help
    );

    println!("");

    println!("Please enter these commands as it is: ");
    println!("Example: cargo run {:?}", VaultCommands::Tasks);
}

/// command redirection logic
///
/// command calls the requested apis
fn vault_commands(command: &[String]) -> std::io::Result<()> {
    match command.get(1) {
        Some(command) => match command.to_lowercase().as_str() {
            // binding to command
            "tasks" => tasks()?,
            "secrets" => secrets(),
            "dashboard" => dashboard(),
            "settings" => settings(),
            "exit" => exit_app(),
            "help" => help(),
            _ => panic!("Command Not Found!"),
        },
        None => println!("Please enter a command"),
    }
    Ok(())
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
