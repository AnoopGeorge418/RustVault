# RustVault

**A local-first, terminal-based personal productivity and secrets management application built with Rust.**

## 1. Project Vision

RustVault is a personal command center that combines task management and secure secret storage in one terminal application.

It provides a single place to organize everyday tasks, manage projects, store sensitive credentials, and access important information without depending on cloud services.

The application starts as a CLI tool and is designed to evolve into a reusable Rust library and eventually a published Cargo crate.

**Core philosophy:**
- Local-first: Your data stays on your machine.
- Privacy-first: Sensitive information is encrypted at rest.
- Terminal-first: Everything is accessible through the command line.
- Modular: Features can evolve independently.
- Extensible: Designed with future integrations and interfaces in mind.

## 2. Core Modules

### Module A: Task Management

A personal task management system for organizing everyday work and projects.

Features:
- Create, edit and delete tasks.
- Mark tasks as pending, in progress or completed.
- Set priorities: low, medium, high and urgent.
- Add descriptions and notes.
- Set due dates and deadlines.
- Organize tasks into projects.
- Add tags and categories.
- Search, filter and sort tasks.
- View upcoming and overdue tasks.
- Track task completion statistics.

### Module B: Secrets Vault

A secure local storage system for sensitive information.

Features:
- Store API tokens and access keys.
- Store passwords and credentials.
- Store GitHub personal access tokens.
- Store database connection strings.
- Store environment variables.
- Store SSH-related credentials and other sensitive configuration.
- Organize secrets using categories and tags.
- Search and retrieve secrets.
- Edit and delete stored secrets.
- Copy secrets to the clipboard.
- Generate secure random passwords.
- Support secret expiration metadata.
- Protect the vault using a master password.

**Security requirements:**
- Encrypt sensitive data at rest.
- Use established cryptographic libraries, never custom cryptographic implementations.
- Use a password-based key derivation function.
- Never log or accidentally print secret values.
- Avoid storing master passwords in plaintext.
- Provide secure vault locking and unlocking.
- Consider clipboard clearing and its limitations.

## 3. Future Module: Knowledge Management

Planned for a later release.

Features:
- Create and manage Markdown notes.
- Organize notes into folders.
- Add tags and categories.
- Search note content.
- Store code snippets.
- Link related notes.
- Import and export notes.
- Attach files and resources.

## 4. Example CLI Experience

**Interactive interface**:

    ╭──────────────────────────────────────╮
    │              🦀 RUSTVAULT            │
    │       Your personal command center   │
    ╰──────────────────────────────────────╯

    1. Tasks
    2. Secrets Vault
    3. Projects
    4. Dashboard
    5. Settings
    6. Exit

**Direct commands**:

    rustvault task create
    rustvault task list
    rustvault task complete 12

    rustvault secret create
    rustvault secret list
    rustvault secret get github-token
    rustvault secret delete github-token

    rustvault dashboard
    rustvault lock
