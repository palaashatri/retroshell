//! `.app` bundle install — SHA-256 integrity verification, extract, atomic move.
//!
//! This cycle verifies a checksum from the catalog against the downloaded archive.
//! Cryptographic signing (ed25519/minisign) is future hardening — not implemented here.

#![allow(dead_code, unused_imports)]

use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CatalogEntry {
    pub name: String,
    pub bundle_id: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug)]
pub enum InstallError {
    Io(String),
    Checksum { expected: String, got: String },
    Extract(String),
    NoDotApp,
}

/// Verify `archive`'s sha256 == `expected` (integrity only; signing is future work).
/// Extract the `.app.tar.gz` into a staging dir and atomically rename the top-level
/// `<Name>.app` into `install_dir`. Returns the installed `<Name>.app` path.
pub fn install_from_archive(
    archive: &Path,
    expected_sha256: &str,
    install_dir: &Path,
) -> Result<PathBuf, InstallError> {
    let got = sha256_hex_file(archive)?;
    let expected = expected_sha256.to_ascii_lowercase();
    if got != expected {
        return Err(InstallError::Checksum { expected, got });
    }

    fs::create_dir_all(install_dir).map_err(|e| InstallError::Io(e.to_string()))?;

    let pid = std::process::id();
    let staging = install_dir.join(format!(".staging-{pid}"));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| InstallError::Io(e.to_string()))?;
    }
    fs::create_dir_all(&staging).map_err(|e| InstallError::Io(e.to_string()))?;

    let extract_result = extract_tar_gz(archive, &staging);
    if let Err(err) = extract_result {
        fs::remove_dir_all(&staging).ok();
        return Err(err);
    }

    let app_dirs: Vec<PathBuf> = fs::read_dir(&staging)
        .map_err(|e| InstallError::Io(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.extension().is_some_and(|ext| ext == "app"))
        .collect();

    if app_dirs.len() != 1 {
        fs::remove_dir_all(&staging).ok();
        return Err(InstallError::NoDotApp);
    }

    let app_path = app_dirs[0].clone();
    let final_name = app_path
        .file_name()
        .ok_or_else(|| InstallError::Io("missing .app name".to_string()))?;
    let final_path = install_dir.join(final_name);

    if final_path.exists() {
        fs::remove_dir_all(&final_path).map_err(|e| InstallError::Io(e.to_string()))?;
    }

    fs::rename(&app_path, &final_path).map_err(|e| InstallError::Io(e.to_string()))?;
    fs::remove_dir_all(&staging).ok();

    Ok(final_path)
}

/// Parse a JSON catalog (array of `CatalogEntry`) from bytes.
pub fn parse_catalog(bytes: &[u8]) -> Result<Vec<CatalogEntry>, InstallError> {
    serde_json::from_slice(bytes).map_err(|e| InstallError::Io(e.to_string()))
}

fn sha256_hex_file(path: &Path) -> Result<String, InstallError> {
    let mut file = File::open(path).map_err(|e| InstallError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| InstallError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn is_safe_tar_path(path: &Path) -> bool {
    !path.is_absolute()
        && path.components().all(|c| {
            !matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), InstallError> {
    let file = File::open(archive).map_err(|e| InstallError::Io(e.to_string()))?;
    let decoder = GzDecoder::new(file);
    let mut tar = Archive::new(decoder);

    for entry in tar
        .entries()
        .map_err(|e| InstallError::Extract(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| InstallError::Extract(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| InstallError::Extract(e.to_string()))?
            .into_owned();
        if !is_safe_tar_path(&path) {
            return Err(InstallError::Extract(format!(
                "unsafe tar path: {}",
                path.display()
            )));
        }
        let unpack_dest = dest.join(&path);
        entry
            .unpack(&unpack_dest)
            .map_err(|e| InstallError::Extract(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tar::Builder;

    fn sha256_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn build_tiny_app_tar_gz(work: &Path) -> (PathBuf, String) {
        let app_dir = work.join("TinyApp.app");
        fs::create_dir_all(app_dir.join("Resources")).unwrap();
        fs::write(
            app_dir.join("Resources").join("Info.toml"),
            "bundle_id=\"com.slopos.tiny\"\nname=\"TinyApp\"\nversion=\"0.1.0\"\nentrypoint=\"bin/tiny\"\n",
        )
        .unwrap();

        let archive_path = work.join("TinyApp.app.tar.gz");
        let file = File::create(&archive_path).unwrap();
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all("TinyApp.app", &app_dir).unwrap();
        let enc = tar.into_inner().unwrap();
        enc.finish().unwrap();

        let bytes = fs::read(&archive_path).unwrap();
        let sha = sha256_bytes(&bytes);
        (archive_path, sha)
    }

    #[test]
    fn install_from_archive_roundtrip() {
        let work = std::env::temp_dir().join(format!("rs_appstore_install_{}", std::process::id()));
        fs::create_dir_all(&work).unwrap();
        let install_dir = work.join("Applications");
        fs::create_dir_all(&install_dir).unwrap();

        let (archive, sha) = build_tiny_app_tar_gz(&work);
        let installed =
            install_from_archive(&archive, &sha, &install_dir).expect("install should succeed");

        assert_eq!(installed, install_dir.join("TinyApp.app"));
        assert!(installed.join("Resources").join("Info.toml").is_file());

        fs::remove_dir_all(&work).ok();
    }

    #[test]
    fn install_from_archive_checksum_mismatch() {
        let work =
            std::env::temp_dir().join(format!("rs_appstore_checksum_{}", std::process::id()));
        fs::create_dir_all(&work).unwrap();
        let install_dir = work.join("Applications");
        fs::create_dir_all(&install_dir).unwrap();

        let (archive, _) = build_tiny_app_tar_gz(&work);
        let err = install_from_archive(
            &archive,
            "0000000000000000000000000000000000000000000000000000000000000000",
            &install_dir,
        )
        .unwrap_err();

        match err {
            InstallError::Checksum { expected, got } => {
                assert_eq!(
                    expected,
                    "0000000000000000000000000000000000000000000000000000000000000000"
                );
                assert_ne!(got, expected);
            }
            other => panic!("expected Checksum error, got {:?}", other),
        }

        fs::remove_dir_all(&work).ok();
    }

    #[test]
    fn parse_catalog_reads_entries() {
        let json = r#"[
            {
                "name": "TextEdit",
                "bundle_id": "com.slopos.textedit",
                "version": "0.1.0",
                "url": "/tmp/TextEdit.app.tar.gz",
                "sha256": "abc123",
                "size": 42
            }
        ]"#;
        let entries = parse_catalog(json.as_bytes()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "TextEdit");
        assert_eq!(entries[0].bundle_id, "com.slopos.textedit");
        assert_eq!(entries[0].size, 42);
    }
}
