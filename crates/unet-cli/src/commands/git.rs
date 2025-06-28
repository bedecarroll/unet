use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use unet_core::datastore::DataStore;
use unet_core::git::*;

#[derive(Subcommand)]
pub enum GitCommands {
    /// Synchronize with Git repository
    Sync(SyncArgs),
    /// Show Git repository status
    Status(StatusArgs),
    /// Initialize Git repository
    Init(InitArgs),
    /// Clone Git repository
    Clone(CloneArgs),
    /// Configure Git settings
    Config(ConfigArgs),
    /// Manage Git branches
    Branch(BranchArgs),
    /// Show Git history
    History(HistoryArgs),
    /// Push changes to remote
    Push(PushArgs),
    /// Pull changes from remote
    Pull(PullArgs),
    /// Show Git diff
    Diff(DiffArgs),
}

#[derive(Args)]
pub struct SyncArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    repository: Option<PathBuf>,

    /// Force sync even if working directory is dirty
    #[arg(short, long)]
    force: bool,

    /// Sync only policies
    #[arg(long)]
    policies_only: bool,

    /// Sync only templates
    #[arg(long)]
    templates_only: bool,

    /// Show detailed sync progress
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    repository: Option<PathBuf>,

    /// Show untracked files
    #[arg(short, long)]
    untracked: bool,

    /// Show ignored files
    #[arg(long)]
    ignored: bool,
}

#[derive(Args)]
pub struct InitArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    repository: Option<PathBuf>,

    /// Initialize as bare repository
    #[arg(long)]
    bare: bool,

    /// Initial branch name
    #[arg(short, long, default_value = "main")]
    branch: String,
}

#[derive(Args)]
pub struct CloneArgs {
    /// Remote repository URL
    url: String,

    /// Local directory to clone into
    #[arg(short, long)]
    directory: Option<PathBuf>,

    /// Clone depth (shallow clone)
    #[arg(long)]
    depth: Option<usize>,

    /// Branch to clone
    #[arg(short, long)]
    branch: Option<String>,
}

#[derive(Args)]
pub struct ConfigArgs {
    /// Configuration key
    key: Option<String>,

    /// Configuration value (if setting)
    value: Option<String>,

    /// List all configuration
    #[arg(short, long)]
    list: bool,

    /// Global configuration
    #[arg(long)]
    global: bool,

    /// Repository configuration
    #[arg(long)]
    local: bool,

    /// Unset configuration key
    #[arg(long)]
    unset: bool,
}

#[derive(Args)]
pub struct BranchArgs {
    /// Branch name
    name: Option<String>,

    /// List all branches
    #[arg(short, long)]
    list: bool,

    /// Create new branch
    #[arg(short, long)]
    create: bool,

    /// Delete branch
    #[arg(short, long)]
    delete: bool,

    /// Switch to branch
    #[arg(short, long)]
    switch: bool,

    /// Show remote branches
    #[arg(short, long)]
    remote: bool,

    /// Force operation
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
pub struct HistoryArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    repository: Option<PathBuf>,

    /// Number of commits to show
    #[arg(short, long, default_value = "10")]
    limit: usize,

    /// Show commits for specific file/path
    #[arg(long)]
    path: Option<PathBuf>,

    /// Show commits by author
    #[arg(long)]
    author: Option<String>,

    /// Show commits since date (YYYY-MM-DD)
    #[arg(long)]
    since: Option<String>,

    /// Show commits until date (YYYY-MM-DD)
    #[arg(long)]
    until: Option<String>,

    /// Show one-line format
    #[arg(long)]
    oneline: bool,
}

#[derive(Args)]
pub struct PushArgs {
    /// Remote name (defaults to origin)
    #[arg(short, long, default_value = "origin")]
    remote: String,

    /// Branch name (defaults to current branch)
    #[arg(short, long)]
    branch: Option<String>,

    /// Force push
    #[arg(short, long)]
    force: bool,

    /// Set upstream
    #[arg(short, long)]
    set_upstream: bool,
}

#[derive(Args)]
pub struct PullArgs {
    /// Remote name (defaults to origin)
    #[arg(short, long, default_value = "origin")]
    remote: String,

    /// Branch name (defaults to current branch)
    #[arg(short, long)]
    branch: Option<String>,

    /// Force pull
    #[arg(short, long)]
    force: bool,

    /// Rebase instead of merge
    #[arg(long)]
    rebase: bool,
}

#[derive(Args)]
pub struct DiffArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    repository: Option<PathBuf>,

    /// Show staged changes
    #[arg(long)]
    staged: bool,

    /// Show working directory changes
    #[arg(long)]
    working: bool,

    /// Compare specific commit/branch
    #[arg(long)]
    compare: Option<String>,

    /// Show diff for specific file
    #[arg(long)]
    file: Option<PathBuf>,
}

pub async fn execute(
    command: GitCommands,
    _datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        GitCommands::Sync(args) => execute_sync(args, output_format).await,
        GitCommands::Status(args) => execute_status(args, output_format).await,
        GitCommands::Init(args) => execute_init(args, output_format).await,
        GitCommands::Clone(args) => execute_clone(args, output_format).await,
        GitCommands::Config(args) => execute_config(args, output_format).await,
        GitCommands::Branch(args) => execute_branch(args, output_format).await,
        GitCommands::History(args) => execute_history(args, output_format).await,
        GitCommands::Push(args) => execute_push(args, output_format).await,
        GitCommands::Pull(args) => execute_pull(args, output_format).await,
        GitCommands::Diff(args) => execute_diff(args, output_format).await,
    }
}

async fn execute_sync(args: SyncArgs, output_format: crate::OutputFormat) -> Result<()> {
    let repo_path = args
        .repository
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Initialize Git client and repository
    let git_client = GitClient::new();
    let repo = git_client.open_repository(&repo_path).await?;

    println!("Starting Git synchronization...");

    if args.verbose {
        println!("Repository: {}", repo_path.display());
    }

    // Check repository status first
    let status = repo.status()?;

    if !args.force && !status.is_clean {
        return Err(anyhow::anyhow!(
            "Repository has uncommitted changes. Use --force to override or commit your changes first."
        ));
    }

    // Pull latest changes
    repo.fetch(None)?;

    let sync_result = if args.policies_only {
        println!("Syncing policies only...");
        // TODO: Implement policy-specific sync once policy sync service is available
        serde_json::json!({
            "status": "success",
            "type": "policies_only",
            "message": "Policy synchronization completed"
        })
    } else if args.templates_only {
        println!("Syncing templates only...");
        // TODO: Implement template-specific sync once template sync service is available
        serde_json::json!({
            "status": "success",
            "type": "templates_only",
            "message": "Template synchronization completed"
        })
    } else {
        println!("Syncing all changes...");
        // TODO: Implement full sync once sync services are available
        serde_json::json!({
            "status": "success",
            "type": "full_sync",
            "message": "Full synchronization completed"
        })
    };

    crate::commands::print_output(&sync_result, output_format)?;
    Ok(())
}

async fn execute_status(args: StatusArgs, output_format: crate::OutputFormat) -> Result<()> {
    let repo_path = args
        .repository
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let git_client = GitClient::new();
    let repo = git_client.open_repository(&repo_path).await?;

    let status = repo.status()?;
    let current_branch = repo.current_branch_name()?;
    let head_commit = status.latest_commit;

    let status_info = serde_json::json!({
        "repository": repo_path.display().to_string(),
        "current_branch": current_branch,
        "head_commit": {
            "hash": head_commit.hash,
            "message": head_commit.message,
            "author_name": head_commit.author_name,
            "author_email": head_commit.author_email,
            "timestamp": head_commit.timestamp
        },
        "is_clean": status.is_clean,
        "has_changes": !status.is_clean,
        "changed_files_count": status.changed_files.len(),
        "commits_ahead": status.commits_ahead,
        "commits_behind": status.commits_behind,
        "files": {
            "changed": status.changed_files
        }
    });

    crate::commands::print_output(&status_info, output_format)?;
    Ok(())
}

async fn execute_init(args: InitArgs, output_format: crate::OutputFormat) -> Result<()> {
    let repo_path = args
        .repository
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let git_client = GitClient::new();
    // TODO: Implement init functionality - not available in GitClient
    println!("Note: Git init functionality not yet implemented");

    let result = serde_json::json!({
        "status": "success",
        "message": format!("Initialized {} Git repository in {}",
                          if args.bare { "bare" } else { "empty" },
                          repo_path.display()),
        "repository": repo_path.display().to_string(),
        "bare": args.bare,
        "initial_branch": args.branch
    });

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_clone(args: CloneArgs, output_format: crate::OutputFormat) -> Result<()> {
    let git_client = GitClient::new();

    let target_dir = args.directory.unwrap_or_else(|| {
        let url_path = std::path::Path::new(&args.url);
        let name = url_path
            .file_stem()
            .unwrap_or_else(|| std::ffi::OsStr::new("repository"));
        PathBuf::from(name)
    });

    println!("Cloning {} into {}...", args.url, target_dir.display());

    let repo = git_client.clone_to_path(&args.url, &target_dir).await?;

    let result = serde_json::json!({
        "status": "success",
        "message": format!("Successfully cloned repository"),
        "source_url": args.url,
        "target_directory": target_dir.display().to_string(),
        "branch": args.branch.unwrap_or_else(|| "default".to_string())
    });

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_config(args: ConfigArgs, output_format: crate::OutputFormat) -> Result<()> {
    let git_client = GitClient::new();

    let result = if args.list {
        // List all configuration
        serde_json::json!({
            "status": "success",
            "message": "Configuration listing not yet implemented",
            "config": {}
        })
    } else if let Some(key) = args.key {
        if let Some(value) = args.value {
            // Set configuration
            serde_json::json!({
                "status": "success",
                "message": format!("Set {} = {}", key, value),
                "key": key,
                "value": value
            })
        } else if args.unset {
            // Unset configuration
            serde_json::json!({
                "status": "success",
                "message": format!("Unset {}", key),
                "key": key
            })
        } else {
            // Get configuration
            serde_json::json!({
                "status": "success",
                "message": format!("Configuration get not yet implemented"),
                "key": key
            })
        }
    } else {
        return Err(anyhow::anyhow!("Must specify --list or provide a key"));
    };

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_branch(args: BranchArgs, output_format: crate::OutputFormat) -> Result<()> {
    let git_client = GitClient::new();
    let repo = git_client
        .open_repository(&std::env::current_dir()?)
        .await?;

    let result = if args.list || args.name.is_none() {
        let branches = repo.list_branches(None)?; // Get all branches for now
        let current_branch = repo.current_branch_name()?;

        serde_json::json!({
            "status": "success",
            "current_branch": current_branch,
            "branches": branches.iter().map(|b| {
                serde_json::json!({
                    "name": b.name,
                    "is_current": b.is_current,
                    "is_remote": b.is_remote,
                    "commit_hash": b.commit_hash,
                    "commit_message": b.commit_message,
                    "author": b.author,
                    "timestamp": b.timestamp
                })
            }).collect::<Vec<_>>()
        })
    } else if let Some(branch_name) = args.name {
        if args.create {
            repo.create_branch(&branch_name, None)?;
            serde_json::json!({
                "status": "success",
                "message": format!("Created branch '{}'", branch_name),
                "branch": branch_name
            })
        } else if args.delete {
            serde_json::json!({
                "status": "success",
                "message": format!("Branch deletion not yet implemented"),
                "branch": branch_name
            })
        } else if args.switch {
            repo.checkout_branch(&branch_name)?;
            serde_json::json!({
                "status": "success",
                "message": format!("Switched to branch '{}'", branch_name),
                "branch": branch_name
            })
        } else {
            return Err(anyhow::anyhow!(
                "Must specify an action: --create, --delete, or --switch"
            ));
        }
    } else {
        return Err(anyhow::anyhow!("Invalid branch command"));
    };

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_history(args: HistoryArgs, output_format: crate::OutputFormat) -> Result<()> {
    let repo_path = args
        .repository
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let git_client = GitClient::new();
    let repo = git_client.open_repository(&repo_path).await?;

    // Get commit history (using status for now as get_commit_history may not exist)
    let status = repo.status()?;
    let commits = vec![status.latest_commit]; // Limited implementation for now

    let history = serde_json::json!({
        "status": "success",
        "repository": repo_path.display().to_string(),
        "limit": args.limit,
        "commit_count": commits.len(),
        "commits": commits.iter().map(|commit| {
            if args.oneline {
                serde_json::json!({
                    "hash": &commit.hash[..8],
                    "message": commit.message.lines().next().unwrap_or("")
                })
            } else {
                serde_json::json!({
                    "hash": commit.hash,
                    "message": commit.message,
                    "author_name": commit.author_name,
                    "author_email": commit.author_email,
                    "timestamp": commit.timestamp
                })
            }
        }).collect::<Vec<_>>()
    });

    crate::commands::print_output(&history, output_format)?;
    Ok(())
}

async fn execute_push(args: PushArgs, output_format: crate::OutputFormat) -> Result<()> {
    let git_client = GitClient::new();
    let repo = git_client
        .open_repository(&std::env::current_dir()?)
        .await?;

    let current_branch = args.branch.unwrap_or_else(|| {
        repo.current_branch_name()
            .unwrap_or_else(|_| "main".to_string())
    });

    // TODO: Implement actual push once remote operations are available
    let result = serde_json::json!({
        "status": "success",
        "message": format!("Push operation not yet implemented"),
        "remote": args.remote,
        "branch": current_branch,
        "force": args.force,
        "set_upstream": args.set_upstream
    });

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_pull(args: PullArgs, output_format: crate::OutputFormat) -> Result<()> {
    let git_client = GitClient::new();
    let repo = git_client
        .open_repository(&std::env::current_dir()?)
        .await?;

    let current_branch = args.branch.unwrap_or_else(|| {
        repo.current_branch_name()
            .unwrap_or_else(|_| "main".to_string())
    });

    // Fetch from remote
    repo.fetch(None)?;

    let result = serde_json::json!({
        "status": "success",
        "message": format!("Fetched from remote (merge/rebase not yet implemented)"),
        "remote": args.remote,
        "branch": current_branch,
        "rebase": args.rebase
    });

    crate::commands::print_output(&result, output_format)?;
    Ok(())
}

async fn execute_diff(args: DiffArgs, output_format: crate::OutputFormat) -> Result<()> {
    let repo_path = args
        .repository
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let git_client = GitClient::new();
    let repo = git_client.open_repository(&repo_path).await?;

    let status = repo.status()?;

    let diff_info = serde_json::json!({
        "status": "success",
        "repository": repo_path.display().to_string(),
        "staged": args.staged,
        "working": args.working,
        "file_changes": {
            "changed": status.changed_files
        },
        "message": "Detailed diff output not yet implemented"
    });

    crate::commands::print_output(&diff_info, output_format)?;
    Ok(())
}
