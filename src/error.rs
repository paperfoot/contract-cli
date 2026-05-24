use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Ambiguous: {0}")]
    Ambiguous(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Render error: {0}")]
    Render(String),

    #[error(transparent)]
    Core(#[from] finance_core::error::CoreError),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidInput(_) | Self::Ambiguous(_) | Self::NotFound(_) => 3,
            Self::Config(_) => 2,
            _ => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
            Self::Ambiguous(_) => "ambiguous",
            Self::Config(_) => "config_error",
            Self::NotFound(_) => "not_found",
            Self::Io(_) => "io_error",
            Self::Db(_) => "db_error",
            Self::Serde(_) => "serde_error",
            Self::Render(_) => "render_error",
            Self::Core(_) => "core_error",
            Self::Other(_) => "other",
        }
    }

    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "Check arguments with: contract --help",
            Self::Ambiguous(_) => "Use the full slug or a more specific identifier",
            Self::Config(_) => "Check config with: contract config show",
            Self::NotFound(_) => "List available entities with: contract <kind> list",
            Self::Io(_) => "Retry the command",
            Self::Db(_) => "Check database integrity: contract doctor",
            Self::Render(_) => "Typst render failed — run: contract doctor",
            _ => "",
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
