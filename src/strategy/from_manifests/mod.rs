use crate::source::DocumentSource;
use crate::strategy::from_manifests::dl::{fetch_meta_manifest, fetch_release_manifests};
use crate::strategy::from_manifests::meta_manifest::MetaManifest;
use crate::strategy::from_manifests::release_manifest::parse_release_manifest;
use crate::strategy::{FetchResources, Strategy};
use crate::{Channel, Release, ReleaseIndex, TResult};

mod dl;
mod meta_manifest;
mod release_manifest;

pub struct FromManifests {
    documents: Vec<DocumentSource>,
}

impl FromManifests {
    #[cfg(test)]
    pub(crate) fn from_documents<I: IntoIterator<Item = DocumentSource>>(iter: I) -> Self {
        Self {
            documents: iter.into_iter().collect(),
        }
    }
}

impl Strategy for FromManifests {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let releases = self
            .documents
            .iter()
            .map(|document| {
                document
                    .load()
                    .and_then(|content| parse_release_manifest(&content).map(Release::new))
            })
            .collect::<TResult<Vec<_>>>()?;

        Ok(ReleaseIndex::new(releases))
    }
}

impl FetchResources for FromManifests {
    fn fetch_channel(channel: Channel) -> TResult<Self> {
        let source = fetch_meta_manifest()?;
        let content = source.load()?;
        let content =
            String::from_utf8(content).map_err(|_| FromManifestsError::ParseMetaManifest)?;

        let meta_manifest = MetaManifest::try_from_str(&content)?;

        let release_manifests = fetch_release_manifests(&meta_manifest, channel)?;

        Ok(Self {
            documents: release_manifests,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FromManifestsError {
    #[error("{0}")]
    DeserializeToml(#[from] toml::de::Error),

    // ...
    #[error("Unable to parse the meta manifest")]
    ParseMetaManifest,

    #[error("Unable to parse manifest date")]
    ParseManifestDate,

    #[error("Unable to parse a manifest source in the meta manifest")]
    ParseManifestSource,

    #[error("{0}")]
    ParseRustVersion(#[from] semver::SemVerError),

    #[error("Unable to find Rust version in release manifest")]
    RustVersionNotFoundInManifest,
}
