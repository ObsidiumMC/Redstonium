use std::fmt;

/// Custom error type for the Rustified launcher
#[derive(Debug)]
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
#[derive(Debug)]
pub enum AuthError {
    /// Microsoft authentication failed
    MicrosoftAuth(String),
    /// Xbox Live authentication failed
    XboxAuth(String),
    /// Minecraft authentication failed
    MinecraftAuth(String),
    /// Game ownership verification failed
    GameOwnership(String),
    /// Profile retrieval failed
    ProfileRetrieval(String),
    /// Token cache operations failed
    CacheError(String),
    /// OAuth flow errors
    OAuthError(String),
}

/// Java-related errors
#[derive(Debug)]
pub enum JavaError {
    /// Java installation not found
    NotFound(String),
    /// Java version parsing failed
    VersionParsing(String),
    /// Java execution failed
    ExecutionFailed(String),
    /// Unsupported Java version
    UnsupportedVersion(String),
}

/// Game launching errors
#[derive(Debug)]
pub enum GameError {
    /// Version not found
    VersionNotFound(String),
    /// Invalid version format
    InvalidVersion(String),
    /// Game preparation failed
    PreparationFailed(String),
    /// Game launch failed
    LaunchFailed(String),
    /// Assets download failed
    AssetsDownload(String),
    /// Libraries download failed
    LibrariesDownload(String),
}

/// Instance management errors
#[derive(Debug)]
pub enum InstanceError {
    /// Instance not found
    NotFound(String),
    /// Instance already exists
    AlreadyExists(String),
    /// Invalid instance configuration
    InvalidConfig(String),
    /// Instance creation failed
    CreationFailed(String),
    /// Instance deletion failed
    DeletionFailed(String),
}

/// File manager errors
#[derive(Debug)]
pub enum FileManagerError {
    /// File download failed
    DownloadFailed(String),
    /// File verification failed
    VerificationFailed(String),
    /// Archive extraction failed
    ExtractionFailed(String),
    /// Directory creation failed
    DirectoryCreation(String),
}

/// Custom result type alias
pub type Result<T> = std::result::Result<T, RustifiedError>;

impl fmt::Display for RustifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_msg = match self {
            Self::Auth(e) => format!("Authentication error: {e}"),
            Self::Io(e) => format!("I/O error: {e}"),
            Self::Network(e) => format!("Network error: {e}"),
            Self::Json(e) => format!("JSON parsing error: {e}"),
            Self::Java(e) => format!("Java error: {e}"),
            Self::Game(e) => format!("Game error: {e}"),
            Self::Instance(e) => format!("Instance error: {e}"),
            Self::FileManager(e) => format!("File manager error: {e}"),
            Self::Generic(msg) => msg.clone(),
        };

        write!(
            f,
            "{error_msg}\n\nIf this error persists, please consider opening an issue at: https://github.com/OmarAfet/rustified/issues"
        )
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MicrosoftAuth(msg) => write!(f, "Microsoft authentication failed: {msg}"),
            Self::XboxAuth(msg) => write!(f, "Xbox Live authentication failed: {msg}"),
            Self::MinecraftAuth(msg) => write!(f, "Minecraft authentication failed: {msg}"),
            Self::GameOwnership(msg) => write!(f, "Game ownership verification failed: {msg}"),
            Self::ProfileRetrieval(msg) => write!(f, "Profile retrieval failed: {msg}"),
            Self::CacheError(msg) => write!(f, "Cache operation failed: {msg}"),
            Self::OAuthError(msg) => write!(f, "OAuth flow failed: {msg}"),
        }
    }
}

impl fmt::Display for JavaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Java installation not found: {msg}"),
            Self::VersionParsing(msg) => write!(f, "Java version parsing failed: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "Java execution failed: {msg}"),
            Self::UnsupportedVersion(msg) => write!(f, "Unsupported Java version: {msg}"),
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionNotFound(msg) => write!(f, "Version not found: {msg}"),
            Self::InvalidVersion(msg) => write!(f, "Invalid version: {msg}"),
            Self::PreparationFailed(msg) => write!(f, "Game preparation failed: {msg}"),
            Self::LaunchFailed(msg) => write!(f, "Game launch failed: {msg}"),
            Self::AssetsDownload(msg) => write!(f, "Assets download failed: {msg}"),
            Self::LibrariesDownload(msg) => write!(f, "Libraries download failed: {msg}"),
        }
    }
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Instance not found: {msg}"),
            Self::AlreadyExists(msg) => write!(f, "Instance already exists: {msg}"),
            Self::InvalidConfig(msg) => write!(f, "Invalid instance configuration: {msg}"),
            Self::CreationFailed(msg) => write!(f, "Instance creation failed: {msg}"),
            Self::DeletionFailed(msg) => write!(f, "Instance deletion failed: {msg}"),
        }
    }
}

impl fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DownloadFailed(msg) => write!(f, "File download failed: {msg}"),
            Self::VerificationFailed(msg) => write!(f, "File verification failed: {msg}"),
            Self::ExtractionFailed(msg) => write!(f, "Archive extraction failed: {msg}"),
            Self::DirectoryCreation(msg) => write!(f, "Directory creation failed: {msg}"),
        }
    }
}

impl std::error::Error for RustifiedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Network(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for AuthError {}
impl std::error::Error for JavaError {}
impl std::error::Error for GameError {}
impl std::error::Error for InstanceError {}
impl std::error::Error for FileManagerError {}

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

impl From<anyhow::Error> for RustifiedError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(format!("Anyhow error: {err}"))
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
    pub fn microsoft_auth(msg: impl Into<String>) -> Self {
        Self::MicrosoftAuth(msg.into())
    }

    pub fn xbox_auth(msg: impl Into<String>) -> Self {
        Self::XboxAuth(msg.into())
    }

    pub fn minecraft_auth(msg: impl Into<String>) -> Self {
        Self::MinecraftAuth(msg.into())
    }

    pub fn game_ownership(msg: impl Into<String>) -> Self {
        Self::GameOwnership(msg.into())
    }

    pub fn profile_retrieval(msg: impl Into<String>) -> Self {
        Self::ProfileRetrieval(msg.into())
    }

    pub fn cache_error(msg: impl Into<String>) -> Self {
        Self::CacheError(msg.into())
    }

    pub fn oauth_error(msg: impl Into<String>) -> Self {
        Self::OAuthError(msg.into())
    }
}

impl JavaError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn version_parsing(msg: impl Into<String>) -> Self {
        Self::VersionParsing(msg.into())
    }

    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    pub fn unsupported_version(msg: impl Into<String>) -> Self {
        Self::UnsupportedVersion(msg.into())
    }
}

impl GameError {
    pub fn version_not_found(msg: impl Into<String>) -> Self {
        Self::VersionNotFound(msg.into())
    }

    pub fn invalid_version(msg: impl Into<String>) -> Self {
        Self::InvalidVersion(msg.into())
    }

    pub fn preparation_failed(msg: impl Into<String>) -> Self {
        Self::PreparationFailed(msg.into())
    }

    pub fn launch_failed(msg: impl Into<String>) -> Self {
        Self::LaunchFailed(msg.into())
    }

    pub fn assets_download(msg: impl Into<String>) -> Self {
        Self::AssetsDownload(msg.into())
    }

    pub fn libraries_download(msg: impl Into<String>) -> Self {
        Self::LibrariesDownload(msg.into())
    }
}

impl InstanceError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    pub fn creation_failed(msg: impl Into<String>) -> Self {
        Self::CreationFailed(msg.into())
    }

    pub fn deletion_failed(msg: impl Into<String>) -> Self {
        Self::DeletionFailed(msg.into())
    }
}

impl FileManagerError {
    pub fn download_failed(msg: impl Into<String>) -> Self {
        Self::DownloadFailed(msg.into())
    }

    pub fn verification_failed(msg: impl Into<String>) -> Self {
        Self::VerificationFailed(msg.into())
    }

    pub fn extraction_failed(msg: impl Into<String>) -> Self {
        Self::ExtractionFailed(msg.into())
    }

    pub fn directory_creation(msg: impl Into<String>) -> Self {
        Self::DirectoryCreation(msg.into())
    }

    #[must_use]
    pub fn version_not_found(msg: &str) -> Self {
        Self::DownloadFailed(format!("Version not found: {msg}"))
    }

    #[must_use]
    pub fn filesystem_error(msg: String) -> Self {
        Self::DirectoryCreation(msg)
    }

    #[must_use]
    pub fn validation_failed(msg: String) -> Self {
        Self::VerificationFailed(msg)
    }
}

/// Extension trait to add context to any Result
pub trait ResultExt<T> {
    /// Add context to a Result (similar to `anyhow::Context`)
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
