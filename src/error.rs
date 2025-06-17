use std::fmt;
use thiserror::Error;

/// Custom error type for the Redstonium launcher
#[derive(Debug, Error)]
pub enum RustifiedError {
    /// Authentication-related errors
    Auth(AuthError),
    /// File system and I/O errors
    Io(std::io::Error),
    /// Network and HTTP errors
    Network(reqwest::Error),
    /// JSON parsing errors
    Json(serde_json::Error),
    /// Java-related errors
    Java(JavaError),
    /// Game launching errors
    Game(GameError),
    /// Instance management errors
    Instance(InstanceError),
    /// File management errors
    FileManager(FileManagerError),
    /// Generic errors with custom messages
    Generic(String),
}

/// Authentication-specific errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Microsoft authentication failed
    #[error("Microsoft authentication failed: {0}")]
    MicrosoftAuth(String),
    /// Xbox Live authentication failed
    #[error("Xbox Live authentication failed: {0}")]
    XboxAuth(String),
    /// Minecraft authentication failed
    #[error("Minecraft authentication failed: {0}")]
    MinecraftAuth(String),
    /// Game ownership verification failed
    #[error("Game ownership verification failed: {0}")]
    GameOwnership(String),
    /// Profile retrieval failed
    #[error("Profile retrieval failed: {0}")]
    ProfileRetrieval(String),
    /// Token cache operations failed
    #[error("Cache operation failed: {0}")]
    CacheError(String),
    /// OAuth flow errors
    #[error("OAuth flow failed: {0}")]
    OAuthError(String),
}

/// Java-related errors
#[derive(Debug, Error)]
pub enum JavaError {
    /// Java installation not found
    #[error("Java installation not found: {0}")]
    NotFound(String),
    /// Java version parsing failed
    #[error("Java version parsing failed: {0}")]
    VersionParsing(String),
    /// Java execution failed
    #[error("Java execution failed: {0}")]
    ExecutionFailed(String),
    /// Unsupported Java version
    #[error("Unsupported Java version: {0}")]
    UnsupportedVersion(String),
}

/// Game launching errors
#[derive(Debug, Error)]
pub enum GameError {
    /// Version not found
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    /// Invalid version format
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
    /// Game preparation failed
    #[error("Game preparation failed: {0}")]
    PreparationFailed(String),
    /// Game launch failed
    #[error("Game launch failed: {0}")]
    LaunchFailed(String),
    /// Assets download failed
    #[error("Assets download failed: {0}")]
    AssetsDownload(String),
    /// Libraries download failed
    #[error("Libraries download failed: {0}")]
    LibrariesDownload(String),
}

/// Instance management errors
#[derive(Debug, Error)]
pub enum InstanceError {
    /// Instance not found
    #[error("Instance not found: {0}")]
    NotFound(String),
    /// Instance already exists
    #[error("Instance already exists: {0}")]
    AlreadyExists(String),
    /// Invalid instance configuration
    #[error("Invalid instance configuration: {0}")]
    InvalidConfig(String),
    /// Instance creation failed
    #[error("Instance creation failed: {0}")]
    CreationFailed(String),
    /// Instance deletion failed
    #[error("Instance deletion failed: {0}")]
    DeletionFailed(String),
}

/// File manager errors
#[derive(Debug, Error)]
pub enum FileManagerError {
    /// File download failed
    #[error("File download failed: {0}")]
    DownloadFailed(String),
    /// File verification failed
    #[error("File verification failed: {0}")]
    VerificationFailed(String),
    /// Archive extraction failed
    #[error("Archive extraction failed: {0}")]
    ExtractionFailed(String),
    /// Directory creation failed
    #[error("Directory creation failed: {0}")]
    DirectoryCreation(String),
}

/// Custom result type alias
pub type Result<T> = std::result::Result<T, RustifiedError>;

impl fmt::Display for RustifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_msg = match self {
            Self::Auth(e) => e.to_string(),
            Self::Io(e) => format!("I/O error: {e}"),
            Self::Network(e) => format!("Network error: {e}"),
            Self::Json(e) => format!("JSON parsing error: {e}"),
            Self::Java(e) => e.to_string(),
            Self::Game(e) => e.to_string(),
            Self::Instance(e) => e.to_string(),
            Self::FileManager(e) => e.to_string(),
            Self::Generic(msg) => msg.clone(),
        };

        write!(
            f,
            "{error_msg}\n\nIf this error persists, please consider opening an issue at: https://github.com/OmarAfet/Redstonium/issues"
        )
    }
}

// Conversions from standard library errors
impl From<std::io::Error> for RustifiedError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for RustifiedError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err)
    }
}

impl From<serde_json::Error> for RustifiedError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<std::env::VarError> for RustifiedError {
    fn from(err: std::env::VarError) -> Self {
        Self::Generic(format!("Environment variable error: {err}"))
    }
}

// Conversions from custom error types
impl From<AuthError> for RustifiedError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}

impl From<JavaError> for RustifiedError {
    fn from(err: JavaError) -> Self {
        Self::Java(err)
    }
}

impl From<GameError> for RustifiedError {
    fn from(err: GameError) -> Self {
        Self::Game(err)
    }
}

impl From<InstanceError> for RustifiedError {
    fn from(err: InstanceError) -> Self {
        Self::Instance(err)
    }
}

impl From<FileManagerError> for RustifiedError {
    fn from(err: FileManagerError) -> Self {
        Self::FileManager(err)
    }
}

// Additional From implementations for external library errors
impl From<oauth2::url::ParseError> for RustifiedError {
    fn from(err: oauth2::url::ParseError) -> Self {
        Self::Generic(format!("URL parsing error: {err}"))
    }
}

impl
    From<
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > for RustifiedError
{
    fn from(
        err: oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ) -> Self {
        Self::Auth(AuthError::oauth_error(format!(
            "OAuth token request failed: {err}"
        )))
    }
}

impl From<tokio::task::JoinError> for RustifiedError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Generic(format!("Task join error: {err}"))
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for RustifiedError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::Generic(format!("Channel receive error: {err}"))
    }
}

impl From<Box<dyn std::error::Error>> for RustifiedError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<zip::result::ZipError> for RustifiedError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::FileManager(FileManagerError::ExtractionFailed(format!(
            "Zip operation failed: {err}"
        )))
    }
}

// Convenience methods for creating errors
impl RustifiedError {
    /// Create a generic error with a custom message
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Add context to an error (similar to `anyhow::Context`)
    #[must_use]
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let context_msg = context.into();
        match self {
            Self::Generic(msg) => Self::Generic(format!("{context_msg}: {msg}")),
            _ => Self::Generic(format!("{context_msg}: {self}")),
        }
    }
}

impl AuthError {
    /// Create a new Microsoft authentication error
    pub fn microsoft_auth(msg: impl Into<String>) -> Self {
        Self::MicrosoftAuth(msg.into())
    }

    /// Create a new Xbox Live authentication error
    pub fn xbox_auth(msg: impl Into<String>) -> Self {
        Self::XboxAuth(msg.into())
    }

    /// Create a new Minecraft authentication error
    pub fn minecraft_auth(msg: impl Into<String>) -> Self {
        Self::MinecraftAuth(msg.into())
    }

    /// Create a new game ownership verification error
    pub fn game_ownership(msg: impl Into<String>) -> Self {
        Self::GameOwnership(msg.into())
    }

    /// Create a new profile retrieval error
    pub fn profile_retrieval(msg: impl Into<String>) -> Self {
        Self::ProfileRetrieval(msg.into())
    }

    /// Create a new cache operation error
    pub fn cache_error(msg: impl Into<String>) -> Self {
        Self::CacheError(msg.into())
    }

    /// Create a new OAuth flow error
    pub fn oauth_error(msg: impl Into<String>) -> Self {
        Self::OAuthError(msg.into())
    }
}

impl JavaError {
    /// Create a new Java installation not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new Java version parsing error
    pub fn version_parsing(msg: impl Into<String>) -> Self {
        Self::VersionParsing(msg.into())
    }

    /// Create a new Java execution failed error
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    /// Create a new unsupported Java version error
    pub fn unsupported_version(msg: impl Into<String>) -> Self {
        Self::UnsupportedVersion(msg.into())
    }
}

impl GameError {
    /// Create a new game version not found error
    pub fn version_not_found(msg: impl Into<String>) -> Self {
        Self::VersionNotFound(msg.into())
    }

    /// Create a new invalid version format error
    pub fn invalid_version(msg: impl Into<String>) -> Self {
        Self::InvalidVersion(msg.into())
    }

    /// Create a new game preparation failed error
    pub fn preparation_failed(msg: impl Into<String>) -> Self {
        Self::PreparationFailed(msg.into())
    }

    /// Create a new game launch failed error
    pub fn launch_failed(msg: impl Into<String>) -> Self {
        Self::LaunchFailed(msg.into())
    }

    /// Create a new assets download failed error
    pub fn assets_download(msg: impl Into<String>) -> Self {
        Self::AssetsDownload(msg.into())
    }

    /// Create a new libraries download failed error
    pub fn libraries_download(msg: impl Into<String>) -> Self {
        Self::LibrariesDownload(msg.into())
    }
}

impl InstanceError {
    /// Create a new instance not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new instance already exists error
    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    /// Create a new invalid instance configuration error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a new instance creation failed error
    pub fn creation_failed(msg: impl Into<String>) -> Self {
        Self::CreationFailed(msg.into())
    }

    /// Create a new instance deletion failed error
    pub fn deletion_failed(msg: impl Into<String>) -> Self {
        Self::DeletionFailed(msg.into())
    }
}

impl FileManagerError {
    /// Create a new file download failed error
    pub fn download_failed(msg: impl Into<String>) -> Self {
        Self::DownloadFailed(msg.into())
    }

    /// Create a new file verification failed error
    pub fn verification_failed(msg: impl Into<String>) -> Self {
        Self::VerificationFailed(msg.into())
    }

    /// Create a new archive extraction failed error
    pub fn extraction_failed(msg: impl Into<String>) -> Self {
        Self::ExtractionFailed(msg.into())
    }

    /// Create a new directory creation failed error
    pub fn directory_creation(msg: impl Into<String>) -> Self {
        Self::DirectoryCreation(msg.into())
    }

    /// Create a new version not found error for file manager
    #[must_use]
    pub fn version_not_found(msg: &str) -> Self {
        Self::DownloadFailed(format!("Version not found: {msg}"))
    }

    /// Create a new filesystem error
    #[must_use]
    pub fn filesystem_error(msg: String) -> Self {
        Self::DirectoryCreation(msg)
    }

    /// Create a new validation failed error
    #[must_use]
    pub fn validation_failed(msg: String) -> Self {
        Self::VerificationFailed(msg)
    }
}

/// Extension trait to add context to any Result
pub trait ResultExt<T> {
    /// Add context to a Result
    ///
    /// # Errors
    ///
    /// Returns an error with additional context if the original Result was an error.
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Add context to a Result with a static string
    ///
    /// # Errors
    ///
    /// Returns an error with additional context if the original Result was an error.
    fn context(self, msg: &'static str) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<RustifiedError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.into().with_context(f()))
    }

    fn context(self, msg: &'static str) -> Result<T> {
        self.map_err(|e| e.into().with_context(msg))
    }
}
