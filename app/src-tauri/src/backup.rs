use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::AppError;
use crate::paths::{Paths, ACTIVE_NAME};
use crate::storage::{self, GroupMeta};

pub fn export_library(paths: &Paths, dest: &Path) -> Result<usize, AppError> {
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut count = 0usize;
    for group in storage::list_groups(paths)? {
        let dir = paths.group_dir(&group);
        for entry in std::fs::read_dir(&dir)?.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let is_slot = name.starts_with("slot_") && name.ends_with(".bin");
            if !is_slot && name != "meta.json" {
                continue;
            }
            let data = std::fs::read(entry.path())?;
            zip.start_file(format!("{group}/{name}"), options)?;
            zip.write_all(&data)?;
            if is_slot {
                count += 1;
            }
        }
    }
    zip.finish()?;
    Ok(count)
}

pub fn import_library(paths: &Paths, src: &Path) -> Result<usize, AppError> {
    let file = std::fs::File::open(src)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::new(format!("That doesn't look like a valid backup file: {e}")))?;

    let mut by_group: BTreeMap<String, (Option<GroupMeta>, Vec<(u32, Vec<u8>)>)> = BTreeMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().to_string();
        let Some((group, filename)) = path.split_once('/') else { continue };
        if group.is_empty() || group == ACTIVE_NAME {
            continue;
        }

        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let bucket = by_group.entry(group.to_string()).or_default();

        if filename == "meta.json" {
            bucket.0 = serde_json::from_slice(&data).ok();
        } else if let Some(slot) = filename
            .strip_prefix("slot_")
            .and_then(|s| s.strip_suffix(".bin"))
            .and_then(|s| s.parse::<u32>().ok())
        {
            bucket.1.push((slot, data));
        }
    }

    let mut imported = 0usize;
    for (_, (meta, slots)) in by_group {
        if slots.is_empty() {
            continue;
        }
        let idx = storage::next_group_index(paths)?;
        let new_group = format!("{idx:03}");
        storage::write_meta(paths, &new_group, &meta.unwrap_or_default())?;
        for (slot, data) in slots {
            std::fs::write(storage::slot_path(paths, &new_group, slot), &data)?;
            imported += 1;
        }
    }

    Ok(imported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_paths() -> (tempfile::TempDir, Paths) {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths {
            saved_dir: tmp.path().join("saved"),
            state_file: tmp.path().join("state.txt"),
            shapes_dir: tmp.path().join("shapes"),
        };
        (tmp, paths)
    }

    fn write_slot(paths: &Paths, group: &str, slot: u32, bytes: &[u8]) {
        std::fs::create_dir_all(paths.group_dir(group)).unwrap();
        std::fs::write(storage::slot_path(paths, group, slot), bytes).unwrap();
    }

    #[test]
    fn backup_then_restore_round_trips_bytes_and_labels_into_new_groups() {
        let (_tmp_src, src_paths) = test_paths();
        write_slot(&src_paths, "001", 1, b"emblem-a-bytes");
        write_slot(&src_paths, "001", 2, b"emblem-b-bytes");
        storage::set_emblem_label(&src_paths, "001", 1, "Dragon").unwrap();

        let zip_dir = tempfile::tempdir().unwrap();
        let zip_path = zip_dir.path().join("backup.zip");
        let exported = export_library(&src_paths, &zip_path).unwrap();
        assert_eq!(exported, 2);

        let (_tmp_dest, dest_paths) = test_paths();
        write_slot(&dest_paths, "001", 1, b"unrelated-existing-emblem");

        let imported = import_library(&dest_paths, &zip_path).unwrap();
        assert_eq!(imported, 2);

        let emblems = storage::list_emblems(&dest_paths).unwrap();
        assert_eq!(emblems.len(), 3, "existing library should be preserved, not overwritten");

        let restored_group = emblems
            .iter()
            .find(|e| e.label == "Dragon")
            .expect("restored labeled emblem should exist")
            .group
            .clone();
        assert_ne!(restored_group, "001", "restore should not collide with existing group names");
        assert_eq!(
            std::fs::read(storage::slot_path(&dest_paths, &restored_group, 1)).unwrap(),
            b"emblem-a-bytes"
        );
    }

    #[test]
    fn backup_of_empty_library_produces_zero_count() {
        let (_tmp, paths) = test_paths();
        let zip_dir = tempfile::tempdir().unwrap();
        let zip_path = zip_dir.path().join("backup.zip");
        assert_eq!(export_library(&paths, &zip_path).unwrap(), 0);
        assert_eq!(import_library(&paths, &zip_path).unwrap(), 0);
        assert!(storage::list_emblems(&paths).unwrap().is_empty());
    }

    #[test]
    fn meta_without_labels_field_defaults_to_empty_map_on_restore() {
        let (_tmp_src, src_paths) = test_paths();
        write_slot(&src_paths, "005", 1, b"data");
        storage::write_meta(
            &src_paths,
            "005",
            &GroupMeta {
                userhash: Some("u123".to_string()),
                first_captured: Some("2026-01-01 00:00:00".to_string()),
                labels: HashMap::new(),
            },
        )
        .unwrap();

        let zip_dir = tempfile::tempdir().unwrap();
        let zip_path = zip_dir.path().join("backup.zip");
        export_library(&src_paths, &zip_path).unwrap();

        let (_tmp_dest, dest_paths) = test_paths();
        import_library(&dest_paths, &zip_path).unwrap();
        let emblems = storage::list_emblems(&dest_paths).unwrap();
        assert_eq!(emblems.len(), 1);
        assert_eq!(emblems[0].label, "");
        assert_eq!(emblems[0].captured_at, "2026-01-01 00:00:00");
    }
}
