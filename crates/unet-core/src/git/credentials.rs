//! Git credential handling and authentication

use crate::git::types::{GitError, GitResult};
use git2::{Cred, CredentialType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Git authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitCredentials {
    /// No authentication (for public repositories)
    None,
    /// SSH key authentication
    SshKey {
        /// Path to private key file
        private_key_path: PathBuf,
        /// Path to public key file (optional)
        public_key_path: Option<PathBuf>,
        /// Passphrase for the private key (optional)
        passphrase: Option<String>,
        /// Username for SSH authentication
        username: String,
    },
    /// SSH agent authentication
    SshAgent {
        /// Username for SSH authentication
        username: String,
    },
    /// Username and password authentication (HTTPS)
    UserPass {
        /// Username
        username: String,
        /// Password or personal access token
        password: String,
    },
    /// Personal access token (HTTPS)
    Token {
        /// Token value
        token: String,
        /// Username (optional, some services use token as username)
        username: Option<String>,
    },
}

impl GitCredentials {
    /// Create SSH key credentials
    pub fn ssh_key<P: Into<PathBuf>>(
        private_key_path: P,
        username: String,
        passphrase: Option<String>,
    ) -> Self {
        Self::SshKey {
            private_key_path: private_key_path.into(),
            public_key_path: None,
            passphrase,
            username,
        }
    }

    /// Create SSH key credentials with public key
    pub fn ssh_key_pair<P: Into<PathBuf>>(
        private_key_path: P,
        public_key_path: P,
        username: String,
        passphrase: Option<String>,
    ) -> Self {
        Self::SshKey {
            private_key_path: private_key_path.into(),
            public_key_path: Some(public_key_path.into()),
            passphrase,
            username,
        }
    }

    /// Create SSH agent credentials
    pub fn ssh_agent(username: String) -> Self {
        Self::SshAgent { username }
    }

    /// Create username/password credentials
    pub fn user_pass(username: String, password: String) -> Self {
        Self::UserPass { username, password }
    }

    /// Create token credentials
    pub fn token(token: String, username: Option<String>) -> Self {
        Self::Token { token, username }
    }
}

/// Git credential provider trait
pub trait GitCredentialProvider: Send + Sync {
    /// Get credentials for a repository URL
    fn get_credentials(&self, url: &str) -> GitResult<GitCredentials>;

    /// Update credentials for a repository URL
    fn update_credentials(&self, url: &str, credentials: GitCredentials) -> GitResult<()>;

    /// Remove credentials for a repository URL
    fn remove_credentials(&self, url: &str) -> GitResult<()>;
}

/// In-memory credential provider
#[derive(Debug)]
pub struct MemoryCredentialProvider {
    credentials: Arc<std::sync::RwLock<std::collections::HashMap<String, GitCredentials>>>,
}

impl MemoryCredentialProvider {
    /// Create a new memory credential provider
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add credentials for a repository URL
    pub fn add_credentials(&self, url: String, credentials: GitCredentials) {
        let mut creds = self.credentials.write().unwrap();
        creds.insert(url, credentials);
    }
}

impl Default for MemoryCredentialProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCredentialProvider for MemoryCredentialProvider {
    fn get_credentials(&self, url: &str) -> GitResult<GitCredentials> {
        let creds = self.credentials.read().unwrap();
        creds
            .get(url)
            .cloned()
            .ok_or_else(|| GitError::Credentials {
                message: format!("No credentials found for URL: {}", url),
            })
    }

    fn update_credentials(&self, url: &str, credentials: GitCredentials) -> GitResult<()> {
        let mut creds = self.credentials.write().unwrap();
        creds.insert(url.to_string(), credentials);
        Ok(())
    }

    fn remove_credentials(&self, url: &str) -> GitResult<()> {
        let mut creds = self.credentials.write().unwrap();
        creds.remove(url);
        Ok(())
    }
}

/// Convert GitCredentials to git2::Cred for libgit2
pub fn to_git2_credentials(
    credentials: &GitCredentials,
    credential_type: CredentialType,
) -> GitResult<Cred> {
    match credentials {
        GitCredentials::None => Err(GitError::Credentials {
            message: "No credentials provided".to_string(),
        }),
        GitCredentials::SshKey {
            private_key_path,
            public_key_path,
            passphrase,
            username,
        } => {
            if credential_type.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key(
                    username,
                    public_key_path.as_ref().map(|p| p.as_path()),
                    private_key_path.as_path(),
                    passphrase.as_deref(),
                )
                .map_err(|e| GitError::Git2 { source: e })
            } else {
                Err(GitError::Credentials {
                    message: "SSH key credentials not supported for this operation".to_string(),
                })
            }
        }
        GitCredentials::SshAgent { username } => {
            if credential_type.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key_from_agent(username).map_err(|e| GitError::Git2 { source: e })
            } else {
                Err(GitError::Credentials {
                    message: "SSH agent credentials not supported for this operation".to_string(),
                })
            }
        }
        GitCredentials::UserPass { username, password } => {
            if credential_type.contains(CredentialType::USER_PASS_PLAINTEXT) {
                Cred::userpass_plaintext(username, password)
                    .map_err(|e| GitError::Git2 { source: e })
            } else {
                Err(GitError::Credentials {
                    message: "Username/password credentials not supported for this operation"
                        .to_string(),
                })
            }
        }
        GitCredentials::Token { token, username } => {
            if credential_type.contains(CredentialType::USER_PASS_PLAINTEXT) {
                let user = username.as_deref().unwrap_or("token");
                Cred::userpass_plaintext(user, token).map_err(|e| GitError::Git2 { source: e })
            } else {
                Err(GitError::Credentials {
                    message: "Token credentials not supported for this operation".to_string(),
                })
            }
        }
    }
}

/// Credential callback for git2 operations
pub fn create_credential_callback<P>(
    credential_provider: Arc<P>,
    repository_url: String,
) -> impl Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error> + Send + Sync
where
    P: GitCredentialProvider + 'static + ?Sized,
{
    move |_url: &str, _username_from_url: Option<&str>, credential_type: CredentialType| {
        let credentials = credential_provider
            .get_credentials(&repository_url)
            .map_err(|e| git2::Error::from_str(&e.to_string()))?;

        to_git2_credentials(&credentials, credential_type)
            .map_err(|e| git2::Error::from_str(&e.to_string()))
    }
}
