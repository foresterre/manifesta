use crate::source::channel_manifests::dl::{fetch_meta_manifest, fetch_release_manifests};
use crate::source::channel_manifests::meta_manifest::MetaManifest;
use crate::source::channel_manifests::release_manifest::parse_release_manifest;
use crate::source::Document;
use crate::source::{FetchResources, Source};
use crate::{Channel, Release, ReleaseIndex, TResult};
use std::collections::BTreeSet;
use std::iter::FromIterator;

#[cfg(feature = "fetch")]
mod dl;
#[cfg(feature = "source")]
mod meta_manifest;
#[cfg(feature = "source")]
mod release_manifest;

pub struct ChannelManifests {
    documents: Vec<Document>,
}

impl ChannelManifests {
    #[cfg(test)]
    pub(crate) fn from_documents<I: IntoIterator<Item = Document>>(iter: I) -> Self {
        Self {
            documents: iter.into_iter().collect(),
        }
    }
}

#[cfg(feature = "source_channel_manifests")]
impl Source for ChannelManifests {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let releases = self
            .documents
            .iter()
            .map(|document| {
                document
                    .load()
                    .and_then(|content| parse_release_manifest(&content).map(Release::new))
            })
            .collect::<TResult<BTreeSet<_>>>()?;

        Ok(ReleaseIndex::from_iter(releases))
    }
}

#[cfg(feature = "fetch_channel_manifests")]
impl FetchResources for ChannelManifests {
    fn fetch_channel(channel: Channel) -> TResult<Self> {
        let source = fetch_meta_manifest()?;
        let content = source.load()?;
        let content =
            String::from_utf8(content).map_err(|_| ChannelManifestsError::ParseMetaManifest)?;

        let meta_manifest = MetaManifest::try_from_str(&content)?;

        let release_manifests = fetch_release_manifests(&meta_manifest, channel)?;

        Ok(Self {
            documents: release_manifests,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelManifestsError {
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
