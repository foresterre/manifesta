use crate::manifests::{ManifestSource, MetaManifest};
use crate::release_channel::Channel;
use crate::{ManifestaError, TResult};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

const META_MANIFEST: &str = "https://static.rust-lang.org/manifests.txt";
// 1 day timeout for the meta manifest
const META_MANIFEST_STALENESS_TIMEOUT: Duration = Duration::from_secs(86_400);
// 1 year timeout for the individual release manifests (these manifests should not get outdated)
const RELEASE_MANIFEST_STALENESS_TIMEOUT: Duration = Duration::from_secs(31_557_600);

/// Download the meta manifest, unless it exists in the cache and is not stale
pub fn fetch_meta_manifest() -> TResult<DocumentSource> {
    let cache = cache_dir()?;
    let manifest = download_if_not_stale(
        META_MANIFEST,
        &cache,
        "manifests.txt",
        META_MANIFEST_STALENESS_TIMEOUT,
    )?;

    Ok(manifest)
}

/// Download the the release manifests for a certain channel, unless they exists in the cache and
/// are not stale
pub fn fetch_release_manifests(
    meta_manifest: &MetaManifest,
    channel: Channel,
) -> TResult<Vec<DocumentSource>> {
    let sources = meta_manifest.manifests();
    let cache = cache_dir()?;

    let manifests = sources
        .iter()
        .filter(|source| source.channel() == channel)
        .map(|source| {
            let manifest = manifest_file_name(source);

            download_if_not_stale(
                source.url(),
                &cache,
                manifest,
                RELEASE_MANIFEST_STALENESS_TIMEOUT,
            )
        })
        .collect::<TResult<Vec<DocumentSource>>>()?;

    Ok(manifests)
}

/// A DocumentSource represents a location from which a document can be accessed.
#[derive(Debug, Eq, PartialEq)]
pub enum DocumentSource {
    /// To be used when the document is present on disk (e.g. if pulled from the cache),
    ///  or accessible locally.
    LocalPath(PathBuf),
    /// To be used when the document has just been downloaded from a remote.
    /// The `PathBuf` represents the path to which the document contents were written (as cache).
    /// The `Vec<u8>` represents the document contents, so the just downloaded file doesn't have to
    ///  be written to the cache location, and read again.
    RemoteCached(PathBuf, Vec<u8>),
}

impl DocumentSource {
    pub fn load(&self) -> TResult<Vec<u8>> {
        match self {
            Self::LocalPath(path) => Self::read_all_from_disk(&path),
            Self::RemoteCached(_, buffer) => Ok(buffer.to_owned()),
        }
    }

    fn read_all_from_disk(path: &Path) -> TResult<Vec<u8>> {
        let mut reader = BufReader::new(File::open(path)?);

        let mut memory = Vec::with_capacity(DEFAULT_MEMORY_SIZE);
        reader.read_to_end(&mut memory)?;

        Ok(memory)
    }
}

const DEFAULT_MEMORY_SIZE: usize = 4096;

fn download_if_not_stale<P: AsRef<Path>>(
    url: &str,
    cache_dir: &Path,
    manifest: P,
    timeout: Duration,
) -> TResult<DocumentSource> {
    let manifest_path = cache_dir.join(manifest);

    if manifest_path.exists() && !is_stale(&manifest_path, timeout)? {
        return Ok(DocumentSource::LocalPath(manifest_path));
    } else {
        std::fs::create_dir_all(cache_dir)?;
    }

    let response = attohttpc::get(url)
        .header(
            "User-Agent",
            "Manifesta (github.com/foresterre/manifesta/issues)",
        )
        .send()?;

    // write to memory
    let mut memory = Vec::with_capacity(DEFAULT_MEMORY_SIZE);
    response.write_to(&mut memory)?;

    // write memory to disk
    let mut file = std::fs::File::create(&manifest_path)?;
    let mut writer = BufWriter::new(&mut file);
    writer.write_all(&memory)?;

    Ok(DocumentSource::RemoteCached(manifest_path, memory))
}

fn is_stale<P: AsRef<Path>>(manifest: P, timeout: Duration) -> TResult<bool> {
    let metadata = std::fs::metadata(manifest)?;
    let modification = metadata.modified()?;
    let duration = modification.elapsed()?;

    Ok(timeout < duration)
}

fn manifest_file_name(source: &ManifestSource) -> String {
    format!(
        "{}_{}.toml",
        Into::<&'static str>::into(source.channel()),
        source.date()
    )
}

fn cache_dir() -> TResult<PathBuf> {
    let cache = directories_next::ProjectDirs::from("com", "ilumeo", "manifesta")
        .ok_or(ManifestaError::DlCache)?;
    let cache = cache.cache_dir();
    let cache = cache.join("index");

    Ok(cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_meta_manifest() {
        let meta = fetch_meta_manifest();
        assert!(meta.is_ok());
    }

    #[test]
    fn test_fetch_release_manifest_stable() {
        let meta = fetch_meta_manifest().unwrap();
        let meta_manifest =
            MetaManifest::try_from_str(String::from_utf8(meta.load().unwrap()).unwrap()).unwrap();

        let result = fetch_release_manifests(&meta_manifest, Channel::Stable);

        assert!(result.is_ok());
    }
}
