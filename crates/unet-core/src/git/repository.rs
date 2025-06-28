//! Git repository management wrapper

use crate::git::credentials::{GitCredentialProvider, create_credential_callback};
use crate::git::types::{
    BranchInfo, CommitInfo, FileChange, FileStatus, GitError, GitResult, RemoteInfo,
    RepositoryInfo, RepositoryStats, TagInfo,
};
use chrono::{TimeZone, Utc};
use git2::{BranchType, Commit, Repository, RepositoryOpenFlags, Status, StatusOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

/// Repository status information
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// Current branch name
    pub current_branch: String,
    /// Whether the working directory is clean
    pub is_clean: bool,
    /// Files with changes
    pub changed_files: Vec<FileChange>,
    /// Number of commits ahead of remote
    pub commits_ahead: usize,
    /// Number of commits behind remote
    pub commits_behind: usize,
    /// Latest local commit
    pub latest_commit: CommitInfo,
}

/// Git repository wrapper with high-level operations
pub struct GitRepository {
    /// Internal git2 repository
    repo: Repository,
    /// Repository information
    info: RepositoryInfo,
    /// Credential provider
    credential_provider: Arc<dyn GitCredentialProvider>,
}

impl GitRepository {
    /// Open an existing repository
    pub fn open<P: AsRef<Path>>(
        path: P,
        credential_provider: Arc<dyn GitCredentialProvider>,
    ) -> GitResult<Self> {
        let path = path.as_ref();
        let repo = Repository::open_ext(
            path,
            RepositoryOpenFlags::empty(),
            &[] as &[&std::ffi::OsStr],
        )?;

        let info = Self::build_repository_info(&repo, path)?;

        Ok(Self {
            repo,
            info,
            credential_provider,
        })
    }

    /// Clone a repository
    pub fn clone<P: AsRef<Path>>(
        url: &str,
        path: P,
        credential_provider: Arc<dyn GitCredentialProvider>,
    ) -> GitResult<Self> {
        let path = path.as_ref();

        info!("Cloning repository {} to {}", url, path.display());

        // Create clone options with credentials
        let mut builder = git2::build::RepoBuilder::new();

        // Set up credential callback
        let credential_callback =
            create_credential_callback(credential_provider.clone(), url.to_string());

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(credential_callback);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);

        // Clone the repository
        let repo = builder.clone(url, path)?;
        let info = Self::build_repository_info(&repo, path)?;

        info!("Successfully cloned repository to {}", path.display());

        Ok(Self {
            repo,
            info,
            credential_provider,
        })
    }

    /// Get repository information
    pub fn info(&self) -> &RepositoryInfo {
        &self.info
    }

    /// Get current repository status
    pub fn status(&self) -> GitResult<RepositoryStatus> {
        let current_branch = self.current_branch_name()?;
        let changed_files = self.get_changed_files()?;
        let is_clean = changed_files.is_empty();
        let latest_commit = self.get_latest_commit()?;

        // Get ahead/behind counts
        let (commits_ahead, commits_behind) = self.get_ahead_behind_counts()?;

        Ok(RepositoryStatus {
            current_branch,
            is_clean,
            changed_files,
            commits_ahead,
            commits_behind,
            latest_commit,
        })
    }

    /// Get current branch name
    pub fn current_branch_name(&self) -> GitResult<String> {
        let head = self.repo.head()?;
        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else {
            Err(GitError::RepositoryOperation {
                message: "Could not determine current branch name".to_string(),
            })
        }
    }

    /// List all branches
    pub fn list_branches(&self, branch_type: Option<BranchType>) -> GitResult<Vec<BranchInfo>> {
        let mut branches = Vec::new();
        let current_branch = self.current_branch_name().unwrap_or_default();

        let branch_iter = self.repo.branches(branch_type)?;
        for branch_result in branch_iter {
            let (branch, branch_type) = branch_result?;

            if let Some(name) = branch.name()? {
                let is_current = name == current_branch;
                let is_remote = branch_type == BranchType::Remote;

                if let Ok(commit) = branch.get().peel_to_commit() {
                    let commit_info = Self::commit_to_info(&commit)?;

                    branches.push(BranchInfo {
                        name: name.to_string(),
                        is_current,
                        is_remote,
                        commit_hash: commit_info.hash,
                        commit_message: commit_info.message,
                        author: commit_info.author_name,
                        timestamp: commit_info.timestamp,
                    });
                }
            }
        }

        Ok(branches)
    }

    /// Fetch from remote
    pub fn fetch(&self, remote_name: Option<&str>) -> GitResult<()> {
        let remote_name = remote_name.unwrap_or("origin");

        info!("Fetching from remote: {}", remote_name);

        let mut remote = self.repo.find_remote(remote_name)?;

        // Set up credential callback
        let credential_callback =
            create_credential_callback(self.credential_provider.clone(), self.info.url.clone());

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(credential_callback);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;

        info!("Successfully fetched from remote: {}", remote_name);
        Ok(())
    }

    /// Pull changes (fetch + merge)
    pub fn pull(&self, remote_name: Option<&str>, branch_name: Option<&str>) -> GitResult<()> {
        let remote_name = remote_name.unwrap_or("origin");
        let current_branch = self.current_branch_name()?;
        let branch_name = branch_name.unwrap_or(&current_branch);

        info!("Pulling changes from {}/{}", remote_name, branch_name);

        // First fetch
        self.fetch(Some(remote_name))?;

        // Then merge
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);
        let remote_branch_ref = format!("refs/remotes/{}", remote_branch_name);

        let remote_commit_oid = self.repo.refname_to_id(&remote_branch_ref)?;
        let remote_commit = self.repo.find_commit(remote_commit_oid)?;

        // Perform the merge
        let mut index = self.repo.index()?;
        let head_commit = self.repo.head()?.peel_to_commit()?;

        // Check if we're up to date
        if head_commit.id() == remote_commit.id() {
            info!("Already up to date");
            return Ok(());
        }

        // Create annotated commit for merge operations
        let annotated_commit = self.repo.find_annotated_commit(remote_commit_oid)?;

        // Perform merge analysis
        let analysis = self.repo.merge_analysis(&[&annotated_commit])?;

        if analysis.0.is_fast_forward() {
            // Fast-forward merge
            let mut reference = self.repo.head()?;
            reference.set_target(remote_commit.id(), "Fast-forward merge")?;
            self.repo.set_head(reference.name().unwrap())?;
            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            info!("Fast-forward merge completed");
        } else if analysis.0.is_normal() {
            // Normal merge
            self.repo.merge(&[&annotated_commit], None, None)?;

            // Check for conflicts
            if index.has_conflicts() {
                let conflicts: Vec<String> = index
                    .conflicts()?
                    .filter_map(|conflict| {
                        conflict.ok().and_then(|c| {
                            c.our
                                .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
                        })
                    })
                    .collect();

                return Err(GitError::MergeConflict { files: conflicts });
            }

            // Create merge commit
            let tree_id = index.write_tree()?;
            let tree = self.repo.find_tree(tree_id)?;
            let signature = self.repo.signature()?;
            let message = format!("Merge {} into {}", remote_branch_name, branch_name);

            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&head_commit, &remote_commit],
            )?;

            info!("Merge commit created");
        }

        info!("Pull completed successfully");
        Ok(())
    }

    /// Push changes to remote
    pub fn push(&self, remote_name: Option<&str>, branch_name: Option<&str>) -> GitResult<()> {
        let remote_name = remote_name.unwrap_or("origin");
        let current_branch = self.current_branch_name()?;
        let branch_name = branch_name.unwrap_or(&current_branch);

        info!("Pushing {} to {}", branch_name, remote_name);

        let mut remote = self.repo.find_remote(remote_name)?;

        // Set up credential callback
        let credential_callback =
            create_credential_callback(self.credential_provider.clone(), self.info.url.clone());

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(credential_callback);

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        remote.push(&[&refspec], Some(&mut push_options))?;

        info!("Successfully pushed to remote");
        Ok(())
    }

    /// Switch to a different branch
    pub fn checkout_branch(&self, branch_name: &str) -> GitResult<()> {
        info!("Switching to branch: {}", branch_name);

        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let _commit = branch.get().peel_to_commit()?;

        self.repo.set_head(&format!("refs/heads/{}", branch_name))?;
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::default().safe()))?;

        info!("Successfully switched to branch: {}", branch_name);
        Ok(())
    }

    /// Create a new branch
    pub fn create_branch(&self, branch_name: &str, start_point: Option<&str>) -> GitResult<()> {
        info!("Creating branch: {}", branch_name);

        let commit = if let Some(start_point) = start_point {
            let oid = self.repo.refname_to_id(start_point)?;
            self.repo.find_commit(oid)?
        } else {
            self.repo.head()?.peel_to_commit()?
        };

        self.repo.branch(branch_name, &commit, false)?;

        info!("Successfully created branch: {}", branch_name);
        Ok(())
    }

    /// Get repository statistics
    pub fn get_stats(&self) -> GitResult<RepositoryStats> {
        let mut commit_count = 0;
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        for _commit in revwalk {
            commit_count += 1;
        }

        let branches = self.list_branches(Some(BranchType::Local))?;
        let remotes = self.list_remotes()?;

        // Get tag count
        let tag_count = self.repo.tag_names(None)?.len();

        // Get repository size (approximate)
        let repo_path = self.repo.path();
        let size_bytes = walkdir::WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| entry.metadata().ok())
            .map(|metadata| metadata.len())
            .sum();

        Ok(RepositoryStats {
            total_commits: commit_count,
            branch_count: branches.len(),
            tag_count,
            remote_count: remotes.len(),
            size_bytes,
            tracked_files: 0,     // TODO: Calculate this properly
            contributor_count: 0, // TODO: Calculate this properly
        })
    }

    /// List remotes
    pub fn list_remotes(&self) -> GitResult<Vec<RemoteInfo>> {
        let mut remotes = Vec::new();
        let remote_names = self.repo.remotes()?;

        for remote_name in remote_names.iter().flatten() {
            if let Ok(remote) = self.repo.find_remote(remote_name) {
                if let Some(url) = remote.url() {
                    remotes.push(RemoteInfo {
                        name: remote_name.to_string(),
                        url: url.to_string(),
                        is_default: remote_name == "origin",
                    });
                }
            }
        }

        Ok(remotes)
    }

    /// List all tags
    pub fn list_tags(&self) -> GitResult<Vec<TagInfo>> {
        let mut tags = Vec::new();

        self.repo.tag_foreach(|oid, name| {
            if let Some(tag_name) = std::str::from_utf8(name).ok() {
                if tag_name.starts_with("refs/tags/") {
                    let tag_name = &tag_name[10..]; // Remove "refs/tags/" prefix

                    if let Ok(obj) = self.repo.find_object(oid, None) {
                        match obj.kind() {
                            Some(git2::ObjectType::Tag) => {
                                // Annotated tag
                                if let Some(tag) = obj.as_tag() {
                                    let commit_oid = tag.target_id();
                                    let message = tag.message().map(|s| s.to_string());
                                    let author = tag
                                        .tagger()
                                        .map(|sig| sig.name().unwrap_or("").to_string());
                                    let timestamp = tag
                                        .tagger()
                                        .and_then(|sig| {
                                            Utc.timestamp_opt(sig.when().seconds(), 0).single()
                                        })
                                        .unwrap_or_else(Utc::now);

                                    tags.push(TagInfo {
                                        name: tag_name.to_string(),
                                        commit_hash: commit_oid.to_string(),
                                        message,
                                        author,
                                        timestamp,
                                        is_annotated: true,
                                    });
                                }
                            }
                            _ => {
                                // Lightweight tag (points directly to commit)
                                if let Ok(commit) = obj.peel_to_commit() {
                                    let timestamp = Utc
                                        .timestamp_opt(commit.author().when().seconds(), 0)
                                        .single()
                                        .unwrap_or_else(Utc::now);

                                    tags.push(TagInfo {
                                        name: tag_name.to_string(),
                                        commit_hash: oid.to_string(),
                                        message: None,
                                        author: None,
                                        timestamp,
                                        is_annotated: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            true // Continue iteration
        })?;

        // Sort tags by name
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tags)
    }

    /// Create a new tag
    pub fn create_tag(
        &self,
        tag_name: &str,
        target: Option<&str>,
        message: Option<&str>,
    ) -> GitResult<()> {
        info!("Creating tag: {}", tag_name);

        let target_commit = if let Some(target) = target {
            let oid = self.repo.refname_to_id(target)?;
            self.repo.find_commit(oid)?
        } else {
            self.repo.head()?.peel_to_commit()?
        };

        if let Some(message) = message {
            // Create annotated tag
            let signature = self.repo.signature()?;
            self.repo.tag(
                tag_name,
                &target_commit.as_object(),
                &signature,
                message,
                false, // force = false
            )?;
            info!("Created annotated tag: {}", tag_name);
        } else {
            // Create lightweight tag
            self.repo.tag_lightweight(
                tag_name,
                &target_commit.as_object(),
                false, // force = false
            )?;
            info!("Created lightweight tag: {}", tag_name);
        }

        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_name: &str) -> GitResult<()> {
        info!("Deleting tag: {}", tag_name);

        self.repo.tag_delete(tag_name)?;

        info!("Successfully deleted tag: {}", tag_name);
        Ok(())
    }

    /// Get information about a specific tag
    pub fn get_tag(&self, tag_name: &str) -> GitResult<TagInfo> {
        let tags = self.list_tags()?;
        tags.into_iter()
            .find(|tag| tag.name == tag_name)
            .ok_or_else(|| GitError::InvalidReference {
                reference: format!("refs/tags/{}", tag_name),
            })
    }

    /// Create a commit with staged changes
    pub fn commit(
        &self,
        message: &str,
        author_name: Option<&str>,
        author_email: Option<&str>,
    ) -> GitResult<String> {
        info!("Creating commit with message: {}", message);

        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let parent_commit = if let Ok(head) = self.repo.head() {
            Some(head.peel_to_commit()?)
        } else {
            None
        };

        let signature = if let (Some(name), Some(email)) = (author_name, author_email) {
            git2::Signature::now(name, email)?
        } else {
            self.repo.signature()?
        };

        let parents = if let Some(ref parent) = parent_commit {
            vec![parent]
        } else {
            vec![]
        };

        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;

        info!("Created commit: {}", commit_id);
        Ok(commit_id.to_string())
    }

    /// Stage files for commit
    pub fn stage_files(&self, paths: &[&str]) -> GitResult<()> {
        let mut index = self.repo.index()?;

        for path in paths {
            info!("Staging file: {}", path);
            index.add_path(std::path::Path::new(path))?;
        }

        index.write()?;
        info!("Staged {} files", paths.len());
        Ok(())
    }

    /// Stage all modified files
    pub fn stage_all(&self) -> GitResult<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        info!("Staged all modified files");
        Ok(())
    }

    /// Get the current HEAD commit hash
    pub fn get_current_commit_hash(&self) -> GitResult<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Get the latest remote commit hash for a branch
    pub fn get_latest_remote_commit_hash(&self, branch_name: &str) -> GitResult<String> {
        let remote_branch_ref = format!("refs/remotes/origin/{}", branch_name);
        let remote_commit_oid = self.repo.refname_to_id(&remote_branch_ref)?;
        Ok(remote_commit_oid.to_string())
    }

    // Private helper methods

    fn build_repository_info(repo: &Repository, path: &Path) -> GitResult<RepositoryInfo> {
        let current_branch = if let Ok(head) = repo.head() {
            head.shorthand().unwrap_or("HEAD").to_string()
        } else {
            "HEAD".to_string()
        };

        let (url, remote_name) = if let Ok(remote) = repo.find_remote("origin") {
            (remote.url().unwrap_or("").to_string(), "origin".to_string())
        } else {
            ("".to_string(), "origin".to_string())
        };

        Ok(RepositoryInfo {
            url,
            local_path: path.to_path_buf(),
            current_branch,
            remote_name,
            last_sync: None,
            description: None,
        })
    }

    fn get_changed_files(&self) -> GitResult<Vec<FileChange>> {
        let mut changes = Vec::new();
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = self.repo.statuses(Some(&mut status_opts))?;

        for status_entry in statuses.iter() {
            let path = PathBuf::from(status_entry.path().unwrap_or(""));
            let flags = status_entry.status();

            let status = if flags.contains(Status::WT_NEW) || flags.contains(Status::INDEX_NEW) {
                FileStatus::Added
            } else if flags.contains(Status::WT_MODIFIED) || flags.contains(Status::INDEX_MODIFIED)
            {
                FileStatus::Modified
            } else if flags.contains(Status::WT_DELETED) || flags.contains(Status::INDEX_DELETED) {
                FileStatus::Deleted
            } else if flags.contains(Status::WT_RENAMED) || flags.contains(Status::INDEX_RENAMED) {
                FileStatus::Renamed
            } else if flags.contains(Status::CONFLICTED) {
                FileStatus::Conflicted
            } else {
                FileStatus::Unmodified
            };

            changes.push(FileChange {
                path,
                status,
                old_path: None,    // TODO: Handle renames properly
                lines_added: None, // TODO: Calculate diff stats
                lines_deleted: None,
            });
        }

        Ok(changes)
    }

    fn get_ahead_behind_counts(&self) -> GitResult<(usize, usize)> {
        let head_commit = match self.repo.head() {
            Ok(head) => head.peel_to_commit()?,
            Err(_) => return Ok((0, 0)),
        };

        let current_branch = self.current_branch_name()?;
        let remote_branch_ref = format!("refs/remotes/origin/{}", current_branch);

        let remote_commit = match self.repo.refname_to_id(&remote_branch_ref) {
            Ok(oid) => self.repo.find_commit(oid)?,
            Err(_) => return Ok((0, 0)),
        };

        let (ahead, behind) = self
            .repo
            .graph_ahead_behind(head_commit.id(), remote_commit.id())?;
        Ok((ahead, behind))
    }

    fn get_latest_commit(&self) -> GitResult<CommitInfo> {
        let head_commit = self.repo.head()?.peel_to_commit()?;
        Self::commit_to_info(&head_commit)
    }

    fn commit_to_info(commit: &Commit) -> GitResult<CommitInfo> {
        let hash = commit.id().to_string();
        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let timestamp = Utc
            .timestamp_opt(author.when().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        let parents = commit.parent_ids().map(|id| id.to_string()).collect();

        Ok(CommitInfo {
            hash,
            message,
            author_name,
            author_email,
            timestamp,
            parents,
        })
    }
}
