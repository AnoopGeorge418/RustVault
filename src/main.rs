use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use datetime::{ISO, Instant, LocalDateTime};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write, stdin};
use std::path::PathBuf;
use std::process;

fn main() -> io::Result<()> {
    intro_banner();
    let command: Vec<String> = env::args().collect();
    vault_commands(&command)
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
enum TaskPriority {
    #[default]
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TaskData {
    id: u64,
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

struct StoragePaths {
    directory: PathBuf,
    tasks: PathBuf,
    secrets: PathBuf,
}

fn storage_paths() -> io::Result<StoragePaths> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find the user home directory",
            )
        })?;
    let directory = home.join(".rustvault");
    fs::create_dir_all(&directory)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    }

    Ok(StoragePaths {
        tasks: directory.join("tasks.json"),
        secrets: directory.join("secrets.enc"),
        directory,
    })
}

fn write_private_file(path: &PathBuf, bytes: &[u8]) -> io::Result<()> {
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?
    };

    #[cfg(not(unix))]
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

fn load_tasks(path: &PathBuf) -> io::Result<HashMap<u64, TaskData>> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(error) => Err(error),
    }
}

fn save_tasks(path: &PathBuf, tasks: &HashMap<u64, TaskData>) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(tasks)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_private_file(path, &bytes)
}

fn tasks() -> io::Result<()> {
    let paths = storage_paths()?;
    let mut tasks = load_tasks(&paths.tasks)?;
    let mut next_task_id = tasks.keys().max().copied().unwrap_or(0).saturating_add(1);

    loop {
        println!("\nRustVault tasks");
        println!("1. Create   2. View   3. Update   4. Delete");
        println!("5. Dashboard   0. Back");
        let choice = read_input("Command: ")?.to_lowercase();

        match choice.as_str() {
            "1" | "create" => {
                create_task(&mut tasks, &mut next_task_id)?;
                save_tasks(&paths.tasks, &tasks)?;
            }
            "2" | "view" => view_tasks(&tasks),
            "3" | "update" => {
                if update_task(&mut tasks)? {
                    save_tasks(&paths.tasks, &tasks)?;
                }
            }
            "4" | "delete" => {
                if delete_task(&mut tasks)? {
                    save_tasks(&paths.tasks, &tasks)?;
                }
            }
            "5" | "dashboard" => dashboard(&tasks),
            "0" | "back" | "" => break,
            _ => println!("Command not found."),
        }
    }
    Ok(())
}

fn create_task(tasks: &mut HashMap<u64, TaskData>, next_task_id: &mut u64) -> io::Result<()> {
    println!("\nCreating a task");
    let title = read_input("Title: ")?;
    if title.is_empty() {
        println!("Title cannot be empty.");
        return Ok(());
    }

    let description = read_optional_input("Description (optional): ")?;
    let Some(status) = read_status(
        "Status (todo/in progress/on hold/completed/cancelled; blank for todo): ",
        true,
    )?
    else {
        return Ok(());
    };
    let Some(priority) = read_priority("Priority (low/medium/high/urgent; blank for low): ", true)?
    else {
        return Ok(());
    };
    let due_date = read_optional_input("Due date (optional): ")?;
    let start_date = read_optional_input("Start date (optional): ")?;
    let category = read_optional_input("Category (optional): ")?;
    let tags = read_optional_input("Tags (optional): ")?;
    let completed_at = read_optional_input("Completed at (optional): ")?;

    let id = *next_task_id;
    let Some(next_id) = id.checked_add(1) else {
        println!("Task ID limit reached; cannot create another task.");
        return Ok(());
    };
    *next_task_id = next_id;
    let now = current_datetime();
    let task = TaskData {
        id,
        title,
        description,
        status,
        priority,
        due_date,
        start_date,
        category,
        tags,
        created_at: now.clone(),
        updated_at: now,
        completed_at,
    };
    tasks.insert(id, task);
    println!("Task created with ID {id}.");
    Ok(())
}

fn view_tasks(tasks: &HashMap<u64, TaskData>) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    let mut ids: Vec<u64> = tasks.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(task) = tasks.get(&id) {
            println!("\n{task:#?}");
        }
    }
}

fn update_task(tasks: &mut HashMap<u64, TaskData>) -> io::Result<bool> {
    let Some(id) = select_task_id("Task ID to update: ")? else {
        return Ok(false);
    };
    let Some(task) = tasks.get_mut(&id) else {
        println!("No task found with ID {id}.");
        return Ok(false);
    };

    println!(
        "Update one field: title, description, status, priority, due_date, start_date, category, tags, completed_at"
    );
    println!("For optional fields, enter '-' to clear the value.");
    let field = read_input("Field: ")?.to_lowercase();
    match field.as_str() {
        "title" => {
            let value = read_input("New title: ")?;
            if value.is_empty() {
                println!("Title cannot be empty.");
                return Ok(false);
            }
            task.title = value;
        }
        "description" => update_optional_field(&mut task.description, "New description: ")?,
        "due_date" => update_optional_field(&mut task.due_date, "New due date: ")?,
        "start_date" => update_optional_field(&mut task.start_date, "New start date: ")?,
        "category" => update_optional_field(&mut task.category, "New category: ")?,
        "tags" => update_optional_field(&mut task.tags, "New tags: ")?,
        "completed_at" => {
            update_optional_field(&mut task.completed_at, "New completed-at value: ")?
        }
        "status" => match read_status("New status: ", false)? {
            Some(status) => task.status = status,
            None => return Ok(false),
        },
        "priority" => match read_priority("New priority: ", false)? {
            Some(priority) => task.priority = priority,
            None => return Ok(false),
        },
        _ => {
            println!("Unknown task field.");
            return Ok(false);
        }
    }
    task.updated_at = current_datetime();
    println!("Task {id} updated.");
    Ok(true)
}

fn delete_task(tasks: &mut HashMap<u64, TaskData>) -> io::Result<bool> {
    let Some(id) = select_task_id("Task ID to delete: ")? else {
        return Ok(false);
    };
    if tasks.remove(&id).is_some() {
        println!("Task {id} deleted.");
        Ok(true)
    } else {
        println!("No task found with ID {id}.");
        Ok(false)
    }
}

fn select_task_id(prompt: &str) -> io::Result<Option<u64>> {
    let input = read_input(prompt)?;
    match input.parse::<u64>() {
        Ok(id) => Ok(Some(id)),
        Err(_) => {
            println!("Please enter a valid numeric task ID.");
            Ok(None)
        }
    }
}

fn read_status(prompt: &str, default_if_empty: bool) -> io::Result<Option<TaskStatus>> {
    let input = read_input(prompt)?.to_lowercase();
    if input.is_empty() {
        return Ok(if default_if_empty {
            Some(TaskStatus::Todo)
        } else {
            None
        });
    }
    let status = match input.as_str() {
        "todo" => Some(TaskStatus::Todo),
        "in progress" | "inprogress" => Some(TaskStatus::InProgress),
        "on hold" | "onhold" => Some(TaskStatus::OnHold),
        "completed" => Some(TaskStatus::Completed),
        "cancelled" | "canceled" => Some(TaskStatus::Cancelled),
        _ => None,
    };
    if status.is_none() {
        println!("Unknown status.");
    }
    Ok(status)
}

fn read_priority(prompt: &str, default_if_empty: bool) -> io::Result<Option<TaskPriority>> {
    let input = read_input(prompt)?.to_lowercase();
    if input.is_empty() {
        return Ok(if default_if_empty {
            Some(TaskPriority::Low)
        } else {
            None
        });
    }
    let priority = match input.as_str() {
        "low" => Some(TaskPriority::Low),
        "medium" => Some(TaskPriority::Medium),
        "high" => Some(TaskPriority::High),
        "urgent" => Some(TaskPriority::Urgent),
        _ => None,
    };
    if priority.is_none() {
        println!("Unknown priority.");
    }
    Ok(priority)
}

fn dashboard(tasks: &HashMap<u64, TaskData>) {
    let mut todo = 0;
    let mut in_progress = 0;
    let mut on_hold = 0;
    let mut completed = 0;
    let mut cancelled = 0;
    let mut low = 0;
    let mut medium = 0;
    let mut high = 0;
    let mut urgent = 0;

    for task in tasks.values() {
        match task.status {
            TaskStatus::Todo => todo += 1,
            TaskStatus::InProgress => in_progress += 1,
            TaskStatus::OnHold => on_hold += 1,
            TaskStatus::Completed => completed += 1,
            TaskStatus::Cancelled => cancelled += 1,
        }
        match task.priority {
            TaskPriority::Low => low += 1,
            TaskPriority::Medium => medium += 1,
            TaskPriority::High => high += 1,
            TaskPriority::Urgent => urgent += 1,
        }
    }

    println!("\nTask statistics");
    println!("Total: {}", tasks.len());
    println!(
        "Status — Todo: {todo}, In progress: {in_progress}, On hold: {on_hold}, Completed: {completed}, Cancelled: {cancelled}"
    );
    println!("Priority — Low: {low}, Medium: {medium}, High: {high}, Urgent: {urgent}");
}

#[derive(Debug, Deserialize, Serialize)]
struct SecretEntry {
    key: String,
    value: String,
    expires_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedSecrets {
    salt: [u8; 16],
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

struct SecretSession {
    entries: Vec<SecretEntry>,
    salt: [u8; 16],
    password: String,
}

fn secrets() -> io::Result<()> {
    let paths = storage_paths()?;
    loop {
        println!("\nRustVault secrets");
        println!("1. Create   2. View   3. Update   4. Delete   0. Back");
        let choice = read_input("Command: ")?.to_lowercase();
        match choice.as_str() {
            "1" | "create" => create_secret(&paths.secrets)?,
            "2" | "view" => view_secrets(&paths.secrets)?,
            "3" | "update" => update_secret(&paths.secrets)?,
            "4" | "delete" => delete_secret(&paths.secrets)?,
            "0" | "back" | "" => break,
            _ => println!("Command not found."),
        }
    }
    Ok(())
}

fn unlock_secrets(path: &PathBuf, create_if_missing: bool) -> io::Result<Option<SecretSession>> {
    if !path.exists() {
        if !create_if_missing {
            println!("No secrets have been saved yet.");
            return Ok(None);
        }
        let password =
            rpassword::prompt_password("Create a vault passphrase (minimum 8 characters): ")?;
        if password.len() < 8 {
            println!("Passphrase must be at least 8 characters.");
            return Ok(None);
        }
        let confirmation = rpassword::prompt_password("Confirm vault passphrase: ")?;
        if password != confirmation {
            println!("Passphrases did not match.");
            return Ok(None);
        }
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        return Ok(Some(SecretSession {
            entries: Vec::new(),
            salt,
            password,
        }));
    }

    let bytes = fs::read(path)?;
    let encrypted: EncryptedSecrets = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let password = rpassword::prompt_password("Vault passphrase: ")?;
    let key = derive_encryption_key(&password, &encrypted.salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid encryption key"))?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&encrypted.nonce),
            encrypted.ciphertext.as_ref(),
        )
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Incorrect passphrase or corrupted secrets file",
            )
        })?;
    let entries = serde_json::from_slice(&plaintext)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(SecretSession {
        entries,
        salt: encrypted.salt,
        password,
    }))
}

fn derive_encryption_key(password: &str, salt: &[u8; 16]) -> io::Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
    Ok(key)
}

fn save_secrets(path: &PathBuf, session: &SecretSession) -> io::Result<()> {
    let key = derive_encryption_key(&session.password, &session.salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid encryption key"))?;
    let plaintext = serde_json::to_vec(&session.entries)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Could not encrypt secrets"))?;
    let encrypted = EncryptedSecrets {
        salt: session.salt,
        nonce,
        ciphertext,
    };
    let bytes = serde_json::to_vec(&encrypted)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_private_file(path, &bytes)
}

fn prune_expired_secrets(session: &mut SecretSession) -> bool {
    let now = Instant::now().seconds().max(0) as u64;
    let before = session.entries.len();
    session.entries.retain(|entry| entry.expires_at > now);
    before != session.entries.len()
}

fn create_secret(path: &PathBuf) -> io::Result<()> {
    let Some(mut session) = unlock_secrets(path, true)? else {
        return Ok(());
    };
    prune_expired_secrets(&mut session);
    let key = read_input("Secret key/name: ")?;
    if key.is_empty() {
        println!("Secret key cannot be empty.");
        return Ok(());
    }
    if session.entries.iter().any(|entry| entry.key == key) {
        println!("A secret with that key already exists.");
        return Ok(());
    }
    let value = rpassword::prompt_password("Secret value: ")?;
    let expiry_seconds = read_expiry_seconds()?;
    let now = Instant::now().seconds().max(0) as u64;
    session.entries.push(SecretEntry {
        key,
        value,
        expires_at: now.saturating_add(expiry_seconds),
    });
    save_secrets(path, &session)?;
    println!("Secret encrypted and saved.");
    Ok(())
}

fn view_secrets(path: &PathBuf) -> io::Result<()> {
    let Some(mut session) = unlock_secrets(path, false)? else {
        return Ok(());
    };
    let expired_removed = prune_expired_secrets(&mut session);
    if session.entries.is_empty() {
        println!("No unexpired secrets found.");
    } else {
        session.entries.sort_by(|a, b| a.key.cmp(&b.key));
        for entry in &session.entries {
            println!(
                "\nKey: {}\nValue: {}\nExpires at: {}",
                entry.key,
                entry.value,
                format_epoch_datetime(entry.expires_at)
            );
        }
    }
    if expired_removed {
        save_secrets(path, &session)?;
    }
    Ok(())
}

fn update_secret(path: &PathBuf) -> io::Result<()> {
    let Some(mut session) = unlock_secrets(path, false)? else {
        return Ok(());
    };
    prune_expired_secrets(&mut session);
    let key = read_input("Secret key/name to update: ")?;
    let Some(entry) = session.entries.iter_mut().find(|entry| entry.key == key) else {
        println!("No unexpired secret found with that key.");
        return Ok(());
    };
    let value = rpassword::prompt_password("New value (blank keeps current): ")?;
    if !value.is_empty() {
        entry.value = value;
    }
    let expiry = read_input("New expiry duration in seconds (blank keeps current): ")?;
    if !expiry.is_empty() {
        let seconds = match parse_expiry_seconds(&expiry) {
            Ok(seconds) => seconds,
            Err(error) => {
                println!("{error}");
                return Ok(());
            }
        };
        entry.expires_at = (Instant::now().seconds().max(0) as u64).saturating_add(seconds);
    }
    save_secrets(path, &session)?;
    println!("Secret updated and re-encrypted.");
    Ok(())
}

fn delete_secret(path: &PathBuf) -> io::Result<()> {
    let Some(mut session) = unlock_secrets(path, false)? else {
        return Ok(());
    };
    prune_expired_secrets(&mut session);
    let key = read_input("Secret key/name to delete: ")?;
    let before = session.entries.len();
    session.entries.retain(|entry| entry.key != key);
    if session.entries.len() == before {
        println!("No unexpired secret found with that key.");
        return Ok(());
    }
    save_secrets(path, &session)?;
    println!("Secret deleted and encrypted store updated.");
    Ok(())
}

fn read_expiry_seconds() -> io::Result<u64> {
    loop {
        let input = read_input("Expires in how many seconds? ")?;
        match parse_expiry_seconds(&input) {
            Ok(seconds) => return Ok(seconds),
            Err(error) => println!("{error}"),
        }
    }
}

fn parse_expiry_seconds(input: &str) -> Result<u64, &'static str> {
    match input.parse::<u64>() {
        Ok(seconds) if seconds > 0 => Ok(seconds),
        _ => Err("Enter a positive whole number of seconds, for example 3000."),
    }
}

fn settings() -> io::Result<()> {
    let paths = storage_paths()?;
    println!("\nRustVault storage");
    println!("Directory: {}", paths.directory.display());
    println!("Tasks (plain JSON): {}", paths.tasks.display());
    println!("Secrets (encrypted): {}", paths.secrets.display());
    println!(
        "\nThe vault passphrase is not stored. If it is lost, encrypted secrets cannot be recovered."
    );
    Ok(())
}

fn current_datetime() -> String {
    let now = Instant::now();
    format!("{}Z", LocalDateTime::at(now.seconds()).iso())
}

fn format_epoch_datetime(epoch_seconds: u64) -> String {
    format!("{}Z", LocalDateTime::at(epoch_seconds as i64).iso())
}

fn update_optional_field(field: &mut Option<String>, prompt: &str) -> io::Result<()> {
    let input = read_input(prompt)?;
    if input == "-" {
        *field = None;
    } else if !input.is_empty() {
        *field = Some(input);
    }
    Ok(())
}

fn read_optional_input(prompt: &str) -> io::Result<Option<String>> {
    let input = read_input(prompt)?;
    Ok(if input.is_empty() { None } else { Some(input) })
}

fn read_input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    let bytes_read = stdin().read_line(&mut input)?;
    if bytes_read == 0 {
        return Ok(String::new());
    }
    Ok(input.trim().to_string())
}

fn vault_commands(command: &[String]) -> io::Result<()> {
    match command.get(1).map(|value| value.to_lowercase()).as_deref() {
        Some("tasks") => tasks(),
        Some("secrets") => secrets(),
        Some("dashboard") => {
            let paths = storage_paths()?;
            let tasks = load_tasks(&paths.tasks)?;
            dashboard(&tasks);
            Ok(())
        }
        Some("settings") => settings(),
        Some("exit") => exit_app(),
        Some("help") => {
            help();
            Ok(())
        }
        Some(command) => {
            eprintln!("Command not found: {command}");
            Ok(())
        }
        None => {
            help();
            Ok(())
        }
    }
}

fn help() {
    println!("RustVault stores tasks in plain JSON and secrets in an encrypted local file.");
    println!("\nCommands: tasks, secrets, dashboard, settings, help, exit");
    println!("Example: cargo run -- tasks");
}

fn intro_banner() {
    println!("╭───────────────────────────────────────────────╮");
    println!("│               🦀 RUST VAULT                   │");
    println!("│         Your personal command center          │");
    println!("╰───────────────────────────────────────────────╯");
}

fn exit_app() -> io::Result<()> {
    println!("Leaving already 👋");
    process::exit(0)
}
