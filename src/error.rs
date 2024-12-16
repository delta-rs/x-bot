use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("GitHub error: {0}")]
    GitHubError(String),

    #[error("Octocrab error: {0}")]
    OctocrabError(String),

    #[error("Twitter error: {0}")]
    TwitterError(String),

    #[error("Misc error: {0}")]
    Misc(String),
}

impl From<octocrab::GitHubError> for Error {
    fn from(err: octocrab::GitHubError) -> Self {
        Self::GitHubError(err.message)
    }
}

impl From<octocrab::Error> for Error {
    fn from(err: octocrab::Error) -> Self {
        match err {
            octocrab::Error::Encoder { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::GitHub { source, .. } => Self::GitHubError(source.message),
            octocrab::Error::Http { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Hyper { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Installation { backtrace } => {
                Self::OctocrabError(backtrace.to_string())
            }
            octocrab::Error::InvalidHeaderValue { source, .. } => {
                Self::OctocrabError(source.to_string())
            }
            octocrab::Error::InvalidUtf8 { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::JWT { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Json { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Other { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Serde { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::SerdeUrlEncoded { source, .. } => {
                Self::OctocrabError(source.to_string())
            }
            octocrab::Error::Service { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::Uri { source, .. } => Self::OctocrabError(source.to_string()),
            octocrab::Error::UriParse { source, .. } => Self::OctocrabError(source.to_string()),
            _ => Self::Misc(String::from("unknown octocrab error")),
        }
    }
}

impl From<twitter_v2::Error> for Error {
    fn from(err: twitter_v2::Error) -> Self {
        Self::TwitterError(err.to_string())
    }
}
