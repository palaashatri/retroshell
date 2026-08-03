#!/usr/bin/env python3
"""Harden session-runtime ownership against immediate inode reuse."""

from pathlib import Path

path = Path(__file__).resolve().parents[1] / "crates" / "slopos-session" / "src" / "main.rs"
text = path.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    text = text.replace(old, new, 1)


replace_once(
    "use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};",
    "use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};",
    "OpenOptionsExt import",
)

replace_once(
    '''struct SessionRuntime {
    path: PathBuf,
    identity: Option<DirectoryIdentity>,
}
''',
    '''struct SessionRuntime {
    path: PathBuf,
    identity: Option<DirectoryIdentity>,
    /// Open descriptor for an owned directory. Keeping the original directory
    /// inode referenced prevents the filesystem from immediately reusing that
    /// inode after an attacker or test replaces the pathname.
    identity_handle: Option<fs::File>,
}
''',
    "SessionRuntime identity handle",
)

replace_once(
    '''fn directory_identity(path: &Path) -> Result<DirectoryIdentity, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "cannot inspect runtime directory {}: {error}",
            path.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "runtime path {} is not a directory",
            path.display()
        ));
    }
    Ok(DirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}
''',
    '''fn directory_identity_from_metadata(
    metadata: &fs::Metadata,
    display_path: &Path,
) -> Result<DirectoryIdentity, String> {
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "runtime path {} is not a directory",
            display_path.display()
        ));
    }
    Ok(DirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

fn directory_identity(path: &Path) -> Result<DirectoryIdentity, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "cannot inspect runtime directory {}: {error}",
            path.display()
        )
    })?;
    directory_identity_from_metadata(&metadata, path)
}
''',
    "directory identity helper",
)

replace_once(
    '''    fn owned(path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            identity: Some(directory_identity(&path)?),
            path,
        })
    }
''',
    '''    fn owned(path: PathBuf) -> Result<Self, String> {
        let identity_handle = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|error| {
                format!(
                    "cannot hold runtime directory {} open: {error}",
                    path.display()
                )
            })?;
        let identity = directory_identity_from_metadata(
            &identity_handle.metadata().map_err(|error| {
                format!(
                    "cannot inspect held runtime directory {}: {error}",
                    path.display()
                )
            })?,
            &path,
        )?;
        Ok(Self {
            path,
            identity: Some(identity),
            identity_handle: Some(identity_handle),
        })
    }
''',
    "owned runtime descriptor",
)

replace_once(
    '''    fn unowned(path: PathBuf) -> Self {
        Self {
            path,
            identity: None,
        }
    }
''',
    '''    fn unowned(path: PathBuf) -> Self {
        Self {
            path,
            identity: None,
            identity_handle: None,
        }
    }
''',
    "unowned runtime descriptor",
)

replace_once(
    '''    fn still_owns_path(&self) -> bool {
        self.identity
            .is_some_and(|expected| directory_identity(&self.path).ok() == Some(expected))
    }
''',
    '''    fn still_owns_path(&self) -> bool {
        let (Some(expected), Some(handle)) = (self.identity, self.identity_handle.as_ref()) else {
            return false;
        };
        let held_identity = handle
            .metadata()
            .ok()
            .and_then(|metadata| directory_identity_from_metadata(&metadata, &self.path).ok());
        held_identity == Some(expected)
            && directory_identity(&self.path).ok() == Some(expected)
    }
''',
    "runtime ownership verification",
)

replace_once(
    '''        fs::remove_dir_all(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("foreign"), b"keep").unwrap();
        drop(runtime);

        assert_eq!(fs::read(path.join("foreign")).unwrap(), b"keep");
''',
    '''        let original_identity = runtime.identity.expect("owned identity");
        fs::remove_dir_all(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("foreign"), b"keep").unwrap();
        let replacement_identity = directory_identity(&path).unwrap();
        assert_ne!(
            replacement_identity, original_identity,
            "the held directory descriptor must prevent immediate inode reuse"
        );
        drop(runtime);

        assert_eq!(fs::read(path.join("foreign")).unwrap(), b"keep");
''',
    "replacement runtime regression assertion",
)

path.write_text(text)
print("Applied session runtime identity hardening")
