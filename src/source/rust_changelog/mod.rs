use crate::source::rust_changelog::dl::fetch_releases_md;
use crate::source::Document;
use crate::source::{FetchResources, Source};
use crate::{Channel, Release, ReleaseIndex, TResult};

#[cfg(feature = "fetch_rust_changelog")]
pub(in crate::source::rust_changelog) mod dl;

pub struct RustChangelog {
    source: Document,
}

impl RustChangelog {
    #[cfg(test)]
    pub(crate) fn from_document(source: Document) -> Self {
        Self { source }
    }
}

#[cfg(feature = "source_rust_changelog")]
impl Source for RustChangelog {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let contents = self.source.load()?;
        let content = String::from_utf8(contents).map_err(RustChangelogError::UnrecognizedText)?;

        let releases = content
            .lines()
            .filter(|s| s.starts_with("Version"))
            .filter_map(|s| {
                s.split_ascii_whitespace()
                    .nth(1)
                    .and_then(|s| semver::Version::parse(s).map(Release::new).ok())
            });

        Ok(releases.collect())
    }
}

#[cfg(feature = "fetch_rust_changelog")]
impl FetchResources for RustChangelog {
    fn fetch_channel(channel: Channel) -> TResult<Self> {
        if let Channel::Stable = channel {
            let source = fetch_releases_md()?;
            Ok(Self { source })
        } else {
            Err(RustChangelogError::ChannelNotAvailable(channel).into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RustChangelogError {
    #[error("Channel {0} is not available for the releases-md source type")]
    ChannelNotAvailable(Channel),

    #[error("{0}")]
    UnrecognizedText(#[from] std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use crate::source::rust_changelog::RustChangelog;
    use crate::source::Document;
    use crate::source::FetchResources;
    use crate::{dl_test, Release};
    use crate::{Channel, ReleaseIndex};
    use yare::parameterized;

    #[test]
    fn source_dist_index() {
        let path = [
            env!("CARGO_MANIFEST_DIR"),
            "/resources/rust_changelog/RELEASES.md",
        ]
        .join("");
        let strategy = RustChangelog::from_document(Document::LocalPath(path.into()));
        let index = ReleaseIndex::from_source(strategy).unwrap();

        assert!(index.releases().len() > 50);
        assert_eq!(
            index.releases()[0],
            &Release::new(semver::Version::new(1, 50, 0))
        );
    }

    #[parameterized(
        beta = { Channel::Beta },
        nightly = { Channel::Nightly },
    )]
    fn fetch_unsupported_channel(channel: Channel) {
        dl_test!({
            let file = RustChangelog::fetch_channel(channel);
            assert!(file.is_err());
        })
    }

    #[test]
    fn fetch_supported_channel() {
        dl_test!({
            let file = RustChangelog::fetch_channel(Channel::Stable);
            assert!(file.is_ok());
        })
    }
}
