use crate::source::Document;
#[cfg(feature = "source_rust_dist")]
use crate::source::FetchResources;
use crate::source::Source;
use crate::{Channel, Release, ReleaseIndex, TResult};
use regex::{Captures, Regex};
use std::collections::BTreeSet;
use std::iter::FromIterator;

#[cfg(feature = "fetch_rust_dist")]
pub(in crate::source::rust_dist) mod dl;

pub struct RustDist {
    source: Document,
}

impl RustDist {
    #[cfg(test)]
    pub(crate) fn from_document(source: Document) -> Self {
        Self { source }
    }
}

#[cfg(feature = "source_rust_dist")]
lazy_static::lazy_static! {
    static ref MATCHER: Regex =
        Regex::new(r"(?m)^dist/rustc-(?P<major>\d+).(?P<minor>\d+).(?P<patch>\d+)(?:\-(alpha|beta|nightly)(\.\d+))?").unwrap();
}

#[cfg(feature = "source_rust_dist")]
impl Source for RustDist {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let contents = self.source.load()?;
        let content = String::from_utf8(contents).map_err(RustDistError::UnrecognizedText)?;

        let releases = MATCHER
            .captures_iter(&content)
            .map(parse_release)
            .collect::<TResult<BTreeSet<Release>>>()?;

        Ok(ReleaseIndex::from_iter(releases))
    }
}

#[cfg(feature = "source_rust_dist")]
fn parse_release(capture: Captures) -> TResult<Release> {
    let major = capture["major"]
        .parse::<u64>()
        .map_err(RustDistError::UnableToParseNumber)?;
    let minor = capture["minor"]
        .parse::<u64>()
        .map_err(RustDistError::UnableToParseNumber)?;
    let patch = capture["patch"]
        .parse::<u64>()
        .map_err(RustDistError::UnableToParseNumber)?;

    Ok(Release::new(semver::Version::new(major, minor, patch)))
}

#[cfg(feature = "fetch_rust_dist")]
impl FetchResources for RustDist {
    fn fetch_channel(channel: Channel) -> TResult<Self> {
        if let Channel::Stable = channel {
            let source = dl::fetch()?;
            Ok(Self { source })
        } else {
            Err(RustDistError::ChannelNotAvailable(channel).into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RustDistError {
    #[error("Channel {0} is not yet available for the 'DistIndex' source type")]
    ChannelNotAvailable(Channel),

    #[cfg(feature = "source_rust_dist")]
    #[error("{0}")]
    RusotoTlsError(#[from] rusoto_core::request::TlsError),

    #[error("{0}")]
    UnrecognizedText(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    UnableToParseNumber(#[from] std::num::ParseIntError),
}

#[cfg(test)]
mod tests {
    use crate::source::{Document, RustDist};
    use crate::{Release, ReleaseIndex};

    #[test]
    fn source_rust_dist() {
        let path = [
            env!("CARGO_MANIFEST_DIR"),
            "/resources/rust_dist/dist_static-rust-lang-org.txt",
        ]
        .join("");
        let strategy = RustDist::from_document(Document::LocalPath(path.into()));
        let index = ReleaseIndex::from_source(strategy).unwrap();

        // 71 releases including minor releases from 1.0.0 to 1.51.0
        assert_eq!(index.releases().len(), 71);
        assert_eq!(
            index.releases()[0],
            &Release::new(semver::Version::new(1, 51, 0))
        );
        assert_eq!(
            index.releases()[70],
            &Release::new(semver::Version::new(1, 0, 0))
        );
    }
}
