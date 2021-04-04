use crate::source::channel_manifests::ChannelManifestsError;
use crate::source::dist_index::DistIndexError;
use crate::source::rust_changelog::RustChangelogError;
use crate::source::rust_dist::RustDistError;

pub type TResult<T> = Result<T, RustReleasesError>;

/// Top level failure cases for rust-releases
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RustReleasesError {
    #[error("Unable to create or access RustReleases cache")]
    DlCache,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "attohttpc")]
    #[error("{0}")]
    Network(#[from] attohttpc::Error),

    #[error("Release channel '{0}' was not found")]
    NoSuchChannel(String),

    #[error("{0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    // ---------------
    // Source errors
    // ---------------
    #[cfg(any(
        feature = "source_channel_manifests",
        feature = "fetch_channel_manifests"
    ))]
    #[error("{0}")]
    ChannelManifestsError(#[from] ChannelManifestsError),

    #[cfg(feature = "source_dist_index")]
    #[error("{0}")]
    DistIndexError(#[from] DistIndexError),

    #[cfg(any(feature = "source_rust_changelog", feature = "fetch_rust_changelog"))]
    #[error("{0}")]
    RustChangelogError(#[from] RustChangelogError),

    #[cfg(any(feature = "source_rust_dist", feature = "fetch_rust_dist"))]
    #[error("{0}")]
    RustDistError(#[from] RustDistError),
}
