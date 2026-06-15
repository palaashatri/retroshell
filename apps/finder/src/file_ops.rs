use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<SystemTime>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub fn copy_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        copy_dir_all(src, dst)
    } else {
        fs::copy(src, dst).map(|_| ())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::rename(src, dst)
}

pub fn delete_file(path: &Path) -> std::io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let trash_root = PathBuf::from(home).join(".local/share/Trash");
    let trash_files = trash_root.join("files");
    let trash_info = trash_root.join("info");

    fs::create_dir_all(&trash_files)?;
    fs::create_dir_all(&trash_info)?;

    let file_name = path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
    let dest_path = unique_destination(&trash_files, file_name.as_ref());

    fs::rename(path, &dest_path)?;

    let trash_name = dest_path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid trash path")
    })?;
    let info_path = trash_info.join(format!("{}.trashinfo", trash_name.to_string_lossy()));
    let deletion_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format_unix_time(d.as_secs()))
        .unwrap_or_else(|_| "1970-01-01T00:00:00".to_string());
    let info_content = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        path.to_string_lossy(),
        deletion_time
    );
    fs::write(info_path, info_content)?;

    Ok(())
}

#[allow(dead_code)]
pub fn rename_file(path: &Path, new_name: &str) -> std::io::Result<()> {
    let mut new_path = path.to_path_buf();
    new_path.set_file_name(new_name);
    fs::rename(path, new_path)
}

pub fn duplicate_file(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
    let new_path = unique_copy_destination(parent, file_name.as_ref());
    copy_file(path, &new_path)
}

fn unique_destination(parent: &Path, file_name: &Path) -> PathBuf {
    let mut candidate = parent.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = file_name
        .file_stem()
        .unwrap_or(file_name.as_os_str())
        .to_string_lossy();
    let extension = file_name.extension().map(|ext| ext.to_string_lossy());
    for index in 1.. {
        let name = match &extension {
            Some(ext) => format!("{} {}.{}", stem, index, ext),
            None => format!("{} {}", stem, index),
        };
        candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("unbounded iterator should always produce a candidate")
}

fn unique_copy_destination(parent: &Path, file_name: &Path) -> PathBuf {
    let stem = file_name
        .file_stem()
        .unwrap_or(file_name.as_os_str())
        .to_string_lossy();
    let extension = file_name.extension().map(|ext| ext.to_string_lossy());
    for index in 0.. {
        let suffix = if index == 0 {
            " copy".to_string()
        } else {
            format!(" copy {}", index + 1)
        };
        let name = match &extension {
            Some(ext) => format!("{}{}.{}", stem, suffix, ext),
            None => format!("{}{}", stem, suffix),
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("unbounded iterator should always produce a candidate")
}

fn format_unix_time(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let days = days_since_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
}

pub fn create_directory(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

#[allow(dead_code)]
pub fn get_file_info(path: &Path) -> std::io::Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    Ok(FileInfo {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: path.to_path_buf(),
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        modified: metadata.modified().ok(),
    })
}

pub fn list_directory(path: &Path) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations() {
        let temp = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_root = temp.join(format!("retroshell_finder_test_{}", unique_id));

        // 1. Create Directory
        create_directory(&test_root).unwrap();
        assert!(test_root.is_dir());

        let src_file = test_root.join("src.txt");
        fs::write(&src_file, "hello retro").unwrap();

        // 2. Get Info
        let info = get_file_info(&src_file).unwrap();
        assert_eq!(info.name, "src.txt");
        assert_eq!(info.size, 11);
        assert!(!info.is_dir);
        assert_eq!(info.path, src_file);
        assert!(info.modified.is_some());

        // 3. Duplicate
        duplicate_file(&src_file).unwrap();
        let dup_file = test_root.join("src copy.txt");
        assert!(dup_file.exists());
        assert_eq!(fs::read_to_string(&dup_file).unwrap(), "hello retro");

        // 4. Rename
        rename_file(&dup_file, "renamed.txt").unwrap();
        let renamed_file = test_root.join("renamed.txt");
        assert!(renamed_file.exists());
        assert!(!dup_file.exists());

        // 5. Copy file (into subfolder)
        let sub_dir = test_root.join("sub");
        create_directory(&sub_dir).unwrap();
        let dst_file = sub_dir.join("copied.txt");
        copy_file(&renamed_file, &dst_file).unwrap();
        assert!(dst_file.exists());

        // 6. Move file
        let moved_file = sub_dir.join("moved.txt");
        move_file(&dst_file, &moved_file).unwrap();
        assert!(moved_file.exists());
        assert!(!dst_file.exists());

        // 7. List Directory
        let entries = list_directory(&test_root).unwrap();
        for entry in &entries {
            assert!(!entry.name.is_empty());
            let _ = entry.path.exists();
            let _ = entry.size;
            let _ = entry.is_dir;
        }
        let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&"src.txt".to_string()));
        assert!(names.contains(&"renamed.txt".to_string()));
        assert!(names.contains(&"sub".to_string()));

        // 8. Delete / Trash
        let old_home = std::env::var("HOME");
        std::env::set_var("HOME", &test_root);

        let trash_target = test_root.join("to_trash.txt");
        fs::write(&trash_target, "trash me").unwrap();
        delete_file(&trash_target).unwrap();

        assert!(!trash_target.exists());
        let trashed_file = test_root.join(".local/share/Trash/files/to_trash.txt");
        assert!(trashed_file.exists());

        if let Ok(val) = old_home {
            std::env::set_var("HOME", val);
        } else {
            std::env::remove_var("HOME");
        }

        // Clean up test directory
        let _ = fs::remove_dir_all(&test_root);
    }
}
