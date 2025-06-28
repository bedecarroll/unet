//! Git integration types and error handling

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Git integration error types
#[derive(Error, Debug)]
pub enum GitError {
    /// Git repository operation failed
    #[error("Git operation failed: {message}")]
    RepositoryOperation { message: String },

    /// Git credential error
    #[error("Git credential error: {message}")]
    Credentials { message: String },

    /// Git authentication failed
    #[error("Git authentication failed for repository: {repository}")]
    Authentication { repository: String },

    /// Git repository not found
    #[error("Git repository not found at path: {path}")]
    RepositoryNotFound { path: String },

    /// Git network error
    #[error("Git network error: {message}")]
    Network { message: String },

    /// Git merge conflict
    #[error("Git merge conflict in files: {files:?}")]
    MergeConflict { files: Vec<String> },

    /// Invalid Git reference
    #[error("Invalid Git reference: {reference}")]
    InvalidReference { reference: String },

    /// IO error during Git operations
    #[error("IO error during Git operation: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// git2 library error
    #[error("Git library error: {source}")]
    Git2 {
        #[from]
        source: git2::Error,
    },

    /// Other Git error
    #[error("Git error: {0}")]
    Other(String),
}

/// Result type for Git operations
pub type GitResult<T> = Result<T, GitError>;

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository URL
    pub url: String,
    /// Local path where repository is cloned
    pub local_path: PathBuf,
    /// Current branch name
    pub current_branch: String,
    /// Remote name (typically "origin")
    pub remote_name: String,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Repository description
    pub description: Option<String>,
}

/// Git branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
    /// Whether this is a remote tracking branch
    pub is_remote: bool,
    /// Latest commit hash
    pub commit_hash: String,
    /// Latest commit message
    pub commit_message: String,
    /// Latest commit author
    pub author: String,
    /// Latest commit timestamp
    pub timestamp: DateTime<Utc>,
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Commit author name
    pub author_name: String,
    /// Commit author email
    pub author_email: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Parent commit hashes
    pub parents: Vec<String>,
}

/// Git file status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    /// File is unmodified
    Unmodified,
    /// File is added to index
    Added,
    /// File is modified
    Modified,
    /// File is deleted
    Deleted,
    /// File is renamed
    Renamed,
    /// File is copied
    Copied,
    /// File is untracked
    Untracked,
    /// File is ignored
    Ignored,
    /// File has conflicts
    Conflicted,
}

/// Git file change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path relative to repository root
    pub path: PathBuf,
    /// File status
    pub status: FileStatus,
    /// Old file path (for renames)
    pub old_path: Option<PathBuf>,
    /// Number of lines added
    pub lines_added: Option<usize>,
    /// Number of lines deleted
    pub lines_deleted: Option<usize>,
}

/// Git remote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    /// Remote name
    pub name: String,
    /// Remote URL
    pub url: String,
    /// Whether this is the default fetch remote
    pub is_default: bool,
}

/// Git tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Tagged commit hash
    pub commit_hash: String,
    /// Tag message (for annotated tags)
    pub message: Option<String>,
    /// Tag author (for annotated tags)
    pub author: Option<String>,
    /// Tag timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether this is an annotated tag
    pub is_annotated: bool,
}

/// Git repository statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// Total number of commits
    pub total_commits: usize,
    /// Number of branches
    pub branch_count: usize,
    /// Number of tags
    pub tag_count: usize,
    /// Number of remotes
    pub remote_count: usize,
    /// Repository size in bytes
    pub size_bytes: u64,
    /// Number of files tracked
    pub tracked_files: usize,
    /// Number of contributors
    pub contributor_count: usize,
}
