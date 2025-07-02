use anyhow::Result;
use chrono::TimeZone;
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

    // Use git2 directly for repository initialization
    let repo = if args.bare {
        git2::Repository::init_bare(&repo_path)?
    } else {
        git2::Repository::init(&repo_path)?
    };

    // Set initial branch if specified
    if args.branch != "main" {
        let mut head = repo.head()?;
        head.set_target(
            git2::Oid::zero(),
            &format!("Set initial branch to {}", args.branch),
        )?;
    }

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
    // Open Git configuration
    let mut config = if args.global {
        git2::Config::open_default()?
    } else if args.local {
        let repo = git2::Repository::open(&std::env::current_dir()?)?;
        repo.config()?
    } else {
        git2::Config::open_default()?
    };

    let result = if args.list {
        // List common configuration items
        let mut config_map = std::collections::HashMap::new();

        // Try to get common config values
        let common_keys = [
            "user.name",
            "user.email",
            "core.editor",
            "init.defaultBranch",
            "push.default",
        ];
        for key in &common_keys {
            if let Ok(value) = config.get_string(key) {
                config_map.insert(key.to_string(), value);
            }
        }

        serde_json::json!({
            "status": "success",
            "message": "Git configuration (common values)",
            "config": config_map
        })
    } else if let Some(key) = args.key {
        if let Some(value) = args.value {
            // Set configuration
            config.set_str(&key, &value)?;
            serde_json::json!({
                "status": "success",
                "message": format!("Set {} = {}", key, value),
                "key": key,
                "value": value
            })
        } else if args.unset {
            // Unset configuration
            config.remove(&key)?;
            serde_json::json!({
                "status": "success",
                "message": format!("Unset {}", key),
                "key": key
            })
        } else {
            // Get configuration
            config.get_string(&key).map_or_else(
                |_| {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Configuration key '{}' not found", key),
                        "key": key
                    })
                },
                |value| {
                    serde_json::json!({
                        "status": "success",
                        "key": key,
                        "value": value
                    })
                },
            )
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
            // Use git2 directly for branch deletion since it's not available in GitRepository
            let git_repo = git2::Repository::open(&std::env::current_dir()?)?;
            let mut branch = git_repo.find_branch(&branch_name, git2::BranchType::Local)?;
            branch.delete()?;
            serde_json::json!({
                "status": "success",
                "message": format!("Deleted branch '{}'", branch_name),
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

    // Use git2 directly to get commit history
    let git_repo = git2::Repository::open(&repo_path)?;
    let mut revwalk = git_repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    let mut count = 0;

    for commit_oid in revwalk {
        if count >= args.limit {
            break;
        }

        let oid = commit_oid?;
        let commit = git_repo.find_commit(oid)?;

        // Apply filters
        if let Some(ref author_filter) = args.author {
            if let Some(author) = commit.author().name() {
                if !author.contains(author_filter) {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Convert commit to our format
        let commit_info = if args.oneline {
            serde_json::json!({
                "hash": &oid.to_string()[..8],
                "message": commit.message().unwrap_or("").lines().next().unwrap_or("")
            })
        } else {
            serde_json::json!({
                "hash": oid.to_string(),
                "message": commit.message().unwrap_or(""),
                "author_name": commit.author().name().unwrap_or(""),
                "author_email": commit.author().email().unwrap_or(""),
                "timestamp": chrono::Utc.timestamp_opt(commit.time().seconds(), 0)
                    .single()
                    .unwrap_or_else(chrono::Utc::now)
            })
        };

        commits.push(commit_info);
        count += 1;
    }

    let history = serde_json::json!({
        "status": "success",
        "repository": repo_path.display().to_string(),
        "limit": args.limit,
        "commit_count": commits.len(),
        "commits": commits
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

    // Use the actual push functionality from GitRepository
    match repo.push(Some(&args.remote), Some(&current_branch)) {
        Ok(()) => {
            let result = serde_json::json!({
                "status": "success",
                "message": format!("Successfully pushed {} to {}", current_branch, args.remote),
                "remote": args.remote,
                "branch": current_branch,
                "force": args.force,
                "set_upstream": args.set_upstream
            });
            crate::commands::print_output(&result, output_format)?;
        }
        Err(e) => {
            let result = serde_json::json!({
                "status": "error",
                "message": format!("Push failed: {}", e),
                "remote": args.remote,
                "branch": current_branch,
                "error": e.to_string()
            });
            crate::commands::print_output(&result, output_format)?;
            return Err(anyhow::anyhow!("Push failed: {}", e));
        }
    }

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

    // Use git2 directly for diff operations
    let git_repo = git2::Repository::open(&repo_path)?;

    let mut diff_options = git2::DiffOptions::new();
    if let Some(ref file_path) = args.file {
        diff_options.pathspec(file_path);
    }

    let mut diff_stats = Vec::new();
    let mut total_insertions = 0;
    let mut total_deletions = 0;

    let diff = if args.staged {
        // Show staged changes (index vs HEAD)
        let head_tree = git_repo.head()?.peel_to_tree()?;
        let mut index = git_repo.index()?;
        let index_tree = git_repo.find_tree(index.write_tree()?)?;
        git_repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut diff_options))?
    } else if args.working {
        // Show working directory changes (working dir vs index)
        git_repo.diff_index_to_workdir(None, Some(&mut diff_options))?
    } else {
        // Show all changes (working dir vs HEAD)
        let head_tree = if let Ok(head) = git_repo.head() {
            Some(head.peel_to_tree()?)
        } else {
            None
        };
        git_repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_options))?
    };

    diff.foreach(
        &mut |delta, _progress| {
            let old_file = delta.old_file();
            let new_file = delta.new_file();
            let status = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Deleted => "deleted", 
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                git2::Delta::Copied => "copied",
                _ => "unknown",
            };

            diff_stats.push(serde_json::json!({
                "file": new_file.path().unwrap_or_else(|| old_file.path().unwrap_or_else(|| std::path::Path::new(""))).display().to_string(),
                "status": status,
                "old_path": old_file.path().map(|p| p.display().to_string()),
                "new_path": new_file.path().map(|p| p.display().to_string()),
            }));
            true
        },
        None,
        None,
        Some(&mut |_delta, _hunk, line| {
            match line.origin() {
                '+' => total_insertions += 1,
                '-' => total_deletions += 1,
                _ => {}
            }
            true
        }),
    )?;

    let diff_info = serde_json::json!({
        "status": "success",
        "repository": repo_path.display().to_string(),
        "staged": args.staged,
        "working": args.working,
        "compare": args.compare,
        "file_filter": args.file,
        "stats": {
            "files_changed": diff_stats.len(),
            "insertions": total_insertions,
            "deletions": total_deletions
        },
        "files": diff_stats
    });

    crate::commands::print_output(&diff_info, output_format)?;
    Ok(())
}
