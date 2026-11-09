//! Downloads the Dev journal only when its published SHA-256 changes.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const JOURNAL_URL: &str = "https://raw.githubusercontent.com/Likounsee/PeaceEmu-dev/PeaceEmu/PeaceEmu/dev-actu.md";
const SHA256_URL: &str = "https://raw.githubusercontent.com/Likounsee/PeaceEmu-dev/PeaceEmu/PeaceEmu/dev-actu.sha256";
const MAX_JOURNAL_BYTES: u64 = 512 * 1024;

fn cache_path() -> Option<PathBuf> {
    super::preferences::config_file_path()
        .and_then(|path| path.parent().map(|parent| parent.join("dev-actu.md")))
}

fn sha256(contents: &[u8]) -> String {
    format!("{:x}", Sha256::digest(contents))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Refreshes the local journal cache. Network or filesystem errors preserve the last copy.
pub(crate) fn refresh() {
    let Some(path) = cache_path() else {
        return;
    };

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let remote_hash = ureq::get(SHA256_URL)
            .call()?
            .body_mut()
            .read_to_string()?;
        let remote_hash = remote_hash.trim().to_ascii_lowercase();
        if !valid_sha256(&remote_hash) {
            return Err("the published journal SHA-256 is invalid".into());
        }

        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if !metadata.file_type().is_file() || metadata.len() > MAX_JOURNAL_BYTES {
                return Err("the cached journal is not a regular file or is too large".into());
            }
            let cached = fs::read(&path)?;
            if sha256(&cached) == remote_hash {
                return Ok(());
            }
        }

        let contents = ureq::get(JOURNAL_URL)
            .call()?
            .body_mut()
            .read_to_string()?;
        if contents.len() as u64 > MAX_JOURNAL_BYTES {
            return Err("downloaded journal exceeds the size limit".into());
        }
        if sha256(contents.as_bytes()) != remote_hash {
            return Err("downloaded journal SHA-256 does not match the published hash".into());
        }
        let parent = path.parent().ok_or("journal cache has no parent directory")?;
        fs::create_dir_all(parent)?;
        let temporary = path.with_extension("md.tmp");
        fs::write(&temporary, contents)?;
        fs::rename(&temporary, &path)?;
        Ok(())
    })();

    if let Err(error) = result {
        eprintln!("PeaceEmu Dev journal: {error}");
    }
}

/// Returns the last successfully downloaded journal, or an empty string if unavailable.
pub(crate) fn read_cached() -> String {
    let Some(path) = cache_path() else {
        return String::new();
    };
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() && metadata.len() <= MAX_JOURNAL_BYTES => {}
        _ => return String::new(),
    }
    fs::read_to_string(path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_the_standard_known_vector() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn published_hash_must_be_a_64_character_hex_digest() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256("not-a-hash"));
        assert!(!valid_sha256(&"g".repeat(64)));
    }
}
