use crate::{Channel, ReleaseIndex, TResult};

pub use channel_manifests::{ChannelManifests, ChannelManifestsError};
pub use dist_index::{DistIndex, DistIndexError};
pub use rust_changelog::{RustChangelog, RustChangelogError};
pub use rust_dist::{RustDist, RustDistError};

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[cfg(any(
    feature = "source_channel_manifests",
    feature = "fetch_channel_manifests"
))]
pub mod channel_manifests;
#[cfg(feature = "source_rust_dist")]
pub mod dist_index;
#[cfg(any(feature = "source_rust_changelog", feature = "fetch_rust_changelog"))]
pub mod rust_changelog;
#[cfg(any(feature = "source_rust_dist", feature = "fetch_rust_dist"))]
pub mod rust_dist;

/// An implementation of the `Source` trait can be used to build a release index.
pub trait Source {
    fn build_index(&self) -> TResult<ReleaseIndex>;
}

/// An implementation of the `FetchResources` trait can be used to fetch the input data necessary
/// for a [`Source`] implementation to build an index.
///
/// [`Source`]: crate::source::Source
pub trait FetchResources
where
    Self: Sized,
{
    fn fetch_channel(channel: Channel) -> TResult<Self>;
}

/// Pre-allocated amount of memory for vectors, in case we don't have an idea how much we'll need.
pub(crate) const DEFAULT_MEMORY_SIZE: usize = 4096;

/// A `Document` represents a resource which can be used as an input to construct a `ReleaseIndex`.
#[derive(Debug, Eq, PartialEq)]
pub enum Document {
    /// To be used when the document is present on disk (e.g. if pulled from the cache),
    ///  or accessible locally.
    LocalPath(PathBuf),
    /// To be used when the document has just been downloaded from a remote.
    /// The `PathBuf` represents the path to which the document contents were written (as cache).
    /// The `Vec<u8>` represents the document contents, so the just downloaded file doesn't have to
    ///  be written to the cache location, and read again.
    RemoteCached(PathBuf, Vec<u8>),
}

impl Document {
    /// Load a resource
    pub fn load(&self) -> TResult<Vec<u8>> {
        match self {
            Self::LocalPath(path) => Self::read_all_from_disk(&path),
            Self::RemoteCached(_, buffer) => Ok(buffer.to_owned()),
        }
    }

    // Load the input data file from the given path. Only a single file is supported.
    fn read_all_from_disk(path: &Path) -> TResult<Vec<u8>> {
        let mut reader = BufReader::new(File::open(path)?);

        let mut memory = Vec::with_capacity(DEFAULT_MEMORY_SIZE);
        reader.read_to_end(&mut memory)?;

        Ok(memory)
    }
}
