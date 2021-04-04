use crate::source::Document;
use crate::source::Source;
use crate::{Release, ReleaseIndex, TResult};
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::path::Path;

/// A source which parses Rust release data from the S3 index.
/// The data files which are used as input should be obtained by you separately (i.e.
/// [`FetchResources`] is not implemented for [`DistIndex`]). You can download the input data files by
/// using the `aws` cli utility and running: `aws --no-sign-request s3 ls static-rust-lang-org/dist/ > dist_index.txt`
///
/// You may then load the source by creating the [`DistIndex`] and calling the `build_index` method
/// from the `Source` trait.
///
/// ```rust,no_run
/// use rust_releases::source::DistIndex;
/// use rust_releases::Source;
///
/// let source = DistIndex::from_path("dist_index.txt");
/// let index = source.build_index().expect("Unable to build a release index");
/// ```
///
/// Alternatively you can look at [`RustDist`] which also uses the AWS S3 index, but obtains the
/// input data differently. The [`RustDist`] source does include a [`FetchResources`] implementation.
///
/// [`DistIndex`]: crate::source::dist_index::DistIndex
/// [`RustDist`]: crate::source::rust_dist::RustDist
/// [`FetchResources`]: crate::source::FetchResources
pub struct DistIndex {
    source: Document,
}

impl DistIndex {
    /// Creates a `DistIndex` from a path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            source: Document::LocalPath(path.as_ref().to_path_buf()),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_document(source: Document) -> Self {
        Self { source }
    }
}

#[cfg(feature = "source_dist_index")]
impl Source for DistIndex {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let contents = self.source.load()?;
        let content = String::from_utf8(contents).map_err(DistIndexError::UnrecognizedText)?;

        // NB: poor man's parsing for stable releases only
        let versions = content
            .lines()
            .filter(|s| !s.trim().starts_with("PRE"))
            .filter_map(|line| {
                line.split_ascii_whitespace()
                    .nth(3)
                    .filter(|s| s.starts_with("rust-1"))
            })
            .filter_map(|s| s.split('-').nth(1))
            .flat_map(|s| semver::Version::parse(s).map(Release::new))
            .collect::<BTreeSet<_>>();

        Ok(ReleaseIndex::from_iter(versions))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DistIndexError {
    #[error("{0}")]
    UnrecognizedText(#[from] std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use crate::source::dist_index::DistIndex;
    use crate::source::Document;
    use crate::ReleaseIndex;

    #[test]
    fn strategy_dist_index() {
        let expected_version = semver::Version::parse("1.50.0").unwrap();

        let path = [env!("CARGO_MANIFEST_DIR"), "/resources/dist_index/dist.txt"].join("");
        let strategy = DistIndex::from_document(Document::LocalPath(path.into()));
        let index = ReleaseIndex::from_source(strategy).unwrap();

        assert!(index.releases().len() > 50);
        assert_eq!(index.releases()[0].version(), &expected_version);
    }
}
