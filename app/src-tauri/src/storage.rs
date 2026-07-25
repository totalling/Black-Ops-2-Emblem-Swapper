use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::paths::{Paths, ACTIVE_NAME};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userhash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_captured: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmblemInfo {
    pub id: String,
    pub group: String,
    pub slot: u32,
    pub label: String,
    pub captured_at: String,
}

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn read_meta(paths: &Paths, name: &str) -> Result<Option<GroupMeta>, AppError> {
    let path = paths.group_dir(name).join("meta.json");
    match std::fs::read(&path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(meta) => Ok(Some(meta)),
            Err(e) => {
                eprintln!("warning: ignoring invalid {}: {e}", path.display());
                Ok(None)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn write_meta(paths: &Paths, name: &str, meta: &GroupMeta) -> Result<(), AppError> {
    let dir = paths.group_dir(name);
    std::fs::create_dir_all(&dir)?;
    let bytes = serde_json::to_vec(meta)?;
    std::fs::write(dir.join("meta.json"), bytes)?;
    Ok(())
}

pub fn list_groups(paths: &Paths) -> Result<Vec<String>, AppError> {
    std::fs::create_dir_all(&paths.saved_dir)?;
    let mut names: Vec<String> = std::fs::read_dir(&paths.saved_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != ACTIVE_NAME)
        .collect();

    names.sort_by(|a, b| {
        let ka = a.parse::<u64>().ok();
        let kb = b.parse::<u64>().ok();
        match (ka, kb) {
            (Some(na), Some(nb)) => na.cmp(&nb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
    Ok(names)
}

pub fn group_slots(paths: &Paths, name: &str) -> Vec<u32> {
    let dir = paths.group_dir(name);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut slots: Vec<u32> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|name| parse_slot_filename(&name))
        .collect();
    slots.sort_unstable();
    slots
}

fn parse_slot_filename(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("slot_")?.strip_suffix(".bin")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub fn next_group_index(paths: &Paths) -> Result<u32, AppError> {
    let max = list_groups(paths)?
        .iter()
        .filter_map(|n| n.parse::<u32>().ok())
        .max();
    Ok(max.map(|m| m + 1).unwrap_or(1))
}

pub fn get_or_create_capture_group(paths: &Paths, userhash: &str) -> Result<String, AppError> {
    let groups = list_groups(paths)?;
    if let Some(last) = groups.last() {
        if let Some(meta) = read_meta(paths, last)? {
            if meta.userhash.as_deref() == Some(userhash) {
                return Ok(last.clone());
            }
        }
    }
    let idx = next_group_index(paths)?;
    let name = format!("{idx:03}");
    write_meta(
        paths,
        &name,
        &GroupMeta {
            userhash: Some(userhash.to_string()),
            first_captured: Some(now_string()),
            labels: HashMap::new(),
        },
    )?;
    Ok(name)
}

pub fn set_emblem_label(
    paths: &Paths,
    group: &str,
    slot: u32,
    label: &str,
) -> Result<(), AppError> {
    let mut meta = read_meta(paths, group)?.unwrap_or_default();
    if label.is_empty() {
        meta.labels.remove(&slot.to_string());
    } else {
        meta.labels.insert(slot.to_string(), label.to_string());
    }
    write_meta(paths, group, &meta)?;
    Ok(())
}

pub fn list_emblems(paths: &Paths) -> Result<Vec<EmblemInfo>, AppError> {
    let mut emblems = Vec::new();
    for group in list_groups(paths)? {
        let meta = read_meta(paths, &group)?.unwrap_or_default();
        for slot in group_slots(paths, &group) {
            emblems.push(EmblemInfo {
                id: format!("{group}:{slot}"),
                label: meta
                    .labels
                    .get(&slot.to_string())
                    .cloned()
                    .unwrap_or_default(),
                captured_at: meta.first_captured.clone().unwrap_or_default(),
                group: group.clone(),
                slot,
            });
        }
    }
    emblems.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
    Ok(emblems)
}

pub fn slot_path(paths: &Paths, group: &str, slot: u32) -> std::path::PathBuf {
    paths.group_dir(group).join(format!("slot_{slot}.bin"))
}

pub fn find_duplicate_groups(paths: &Paths) -> Result<Vec<Vec<EmblemInfo>>, AppError> {
    let emblems = list_emblems(paths)?;
    let mut by_body: HashMap<Vec<u8>, Vec<EmblemInfo>> = HashMap::new();

    for emblem in emblems {
        let path = slot_path(paths, &emblem.group, emblem.slot);
        let Ok(data) = std::fs::read(&path) else { continue };
        let body = crate::emblem::format::strip_http(&data).to_vec();
        by_body.entry(body).or_default().push(emblem);
    }

    let mut groups: Vec<Vec<EmblemInfo>> = by_body.into_values().filter(|g| g.len() > 1).collect();
    for group in &mut groups {
        group.sort_by(|a, b| a.captured_at.cmp(&b.captured_at));
    }
    groups.sort_by(|a, b| b.len().cmp(&a.len()));
    Ok(groups)
}

pub fn slot_exists(paths: &Paths, group: &str, slot: u32) -> bool {
    Path::new(&slot_path(paths, group, slot)).exists()
}

pub fn delete_emblem(paths: &Paths, group: &str, slot: u32) -> Result<(), AppError> {
    let path = slot_path(paths, group, slot);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let dir = paths.group_dir(group);
    let thumb_prefix = format!("thumb_{slot}_");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with(&thumb_prefix) && name.ends_with(".png") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    if group_slots(paths, group).is_empty() {
        let _ = std::fs::remove_dir_all(&dir);
    } else {
        let mut meta = read_meta(paths, group)?.unwrap_or_default();
        meta.labels.remove(&slot.to_string());
        write_meta(paths, group, &meta)?;
    }
    Ok(())
}

pub fn thumb_cache_path(paths: &Paths, group: &str, slot: u32, size: u32) -> std::path::PathBuf {
    paths.group_dir(group).join(format!("thumb_{slot}_{size}.png"))
}

pub fn export_emblem(paths: &Paths, group: &str, slot: u32, dest: &Path) -> Result<(), AppError> {
    let data = std::fs::read(slot_path(paths, group, slot))?;
    std::fs::write(dest, data)?;
    Ok(())
}

pub fn store_new_emblem(paths: &Paths, data: &[u8], label: Option<&str>) -> Result<EmblemInfo, AppError> {
    let idx = next_group_index(paths)?;
    let group = format!("{idx:03}");
    let slot = 1u32;
    let captured_at = now_string();

    let mut labels = HashMap::new();
    let label = label.map(str::trim).filter(|l| !l.is_empty());
    if let Some(l) = label {
        labels.insert(slot.to_string(), l.to_string());
    }

    write_meta(
        paths,
        &group,
        &GroupMeta {
            userhash: None,
            first_captured: Some(captured_at.clone()),
            labels,
        },
    )?;
    std::fs::write(slot_path(paths, &group, slot), data)?;

    Ok(EmblemInfo {
        id: format!("{group}:{slot}"),
        label: label.unwrap_or("").to_string(),
        captured_at,
        group,
        slot,
    })
}

pub fn import_emblem(
    paths: &Paths,
    src: &Path,
    label: Option<&str>,
) -> Result<EmblemInfo, AppError> {
    let data = std::fs::read(src)?;

    let body = crate::emblem::format::strip_http(&data);
    let expected_len = crate::emblem::format::LAYER_SIZE * crate::emblem::format::NUM_LAYERS;
    if body.len() < expected_len {
        return Err(AppError::new(
            "That file doesn't look like a valid emblem. It's smaller than a full 32-layer capture.",
        ));
    }

    store_new_emblem(paths, &data, label)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        std::fs::write(slot_path(paths, group, slot), bytes).unwrap();
    }

    #[test]
    fn numeric_groups_sort_before_alpha_and_ascending() {
        let (_tmp, paths) = test_paths();
        for name in ["010", "002", "zzz", "001", "_active", "aaa"] {
            std::fs::create_dir_all(paths.group_dir(name)).unwrap();
        }
        assert_eq!(
            list_groups(&paths).unwrap(),
            vec!["001", "002", "010", "aaa", "zzz"]
        );
    }

    #[test]
    fn next_group_index_continues_from_highest_numeric_name() {
        let (_tmp, paths) = test_paths();
        for name in ["001", "007", "not-numeric"] {
            std::fs::create_dir_all(paths.group_dir(name)).unwrap();
        }
        assert_eq!(next_group_index(&paths).unwrap(), 8);

        let (_tmp2, empty_paths) = test_paths();
        assert_eq!(next_group_index(&empty_paths).unwrap(), 1);
    }

    #[test]
    fn capture_group_reuses_same_target_but_starts_new_group_for_new_target() {
        let (_tmp, paths) = test_paths();
        let g1 = get_or_create_capture_group(&paths, "uaaaa").unwrap();
        assert_eq!(g1, "001");

        let g2 = get_or_create_capture_group(&paths, "uaaaa").unwrap();
        assert_eq!(g2, "001");

        let g3 = get_or_create_capture_group(&paths, "ubbbb").unwrap();
        assert_eq!(g3, "002");
    }

    #[test]
    fn corrupted_meta_json_degrades_to_empty_instead_of_erroring() {
        let (_tmp, paths) = test_paths();
        std::fs::create_dir_all(paths.group_dir("001")).unwrap();
        std::fs::write(paths.group_dir("001").join("meta.json"), b"{ not valid json").unwrap();

        let meta = read_meta(&paths, "001").unwrap();
        assert!(meta.is_none());

        write_slot(&paths, "001", 5, b"HTTP/1.1 200 OK\r\n\r\ndata");
        let emblems = list_emblems(&paths).unwrap();
        assert_eq!(emblems.len(), 1);
        assert_eq!(emblems[0].label, "");
    }

    #[test]
    fn set_emblem_label_add_and_clear() {
        let (_tmp, paths) = test_paths();
        write_slot(&paths, "001", 1, b"data");

        set_emblem_label(&paths, "001", 1, "Sam's dragon").unwrap();
        assert_eq!(list_emblems(&paths).unwrap()[0].label, "Sam's dragon");

        set_emblem_label(&paths, "001", 1, "").unwrap();
        assert_eq!(list_emblems(&paths).unwrap()[0].label, "");
    }

    #[test]
    fn list_emblems_sorts_newest_captured_first() {
        let (_tmp, paths) = test_paths();
        write_slot(&paths, "001", 1, b"a");
        write_slot(&paths, "002", 1, b"b");
        write_meta(
            &paths,
            "001",
            &GroupMeta {
                userhash: None,
                first_captured: Some("2026-01-01 10:00:00".to_string()),
                labels: HashMap::new(),
            },
        )
        .unwrap();
        write_meta(
            &paths,
            "002",
            &GroupMeta {
                userhash: None,
                first_captured: Some("2026-06-01 10:00:00".to_string()),
                labels: HashMap::new(),
            },
        )
        .unwrap();

        let emblems = list_emblems(&paths).unwrap();
        assert_eq!(emblems[0].group, "002");
        assert_eq!(emblems[1].group, "001");
    }

    fn fake_captured_bytes() -> Vec<u8> {
        let mut data = b"HTTP/1.1 200 OK\r\nContent-Length: 1408\r\n\r\n".to_vec();
        data.extend(std::iter::repeat(0u8).take(crate::emblem::format::LAYER_SIZE * crate::emblem::format::NUM_LAYERS));
        data
    }

    #[test]
    fn export_then_import_round_trips_the_same_bytes() {
        let (_tmp_src, src_paths) = test_paths();
        let original = fake_captured_bytes();
        write_slot(&src_paths, "001", 7, &original);

        let export_dir = tempfile::tempdir().unwrap();
        let export_path = export_dir.path().join("exported.bin");
        export_emblem(&src_paths, "001", 7, &export_path).unwrap();
        assert_eq!(std::fs::read(&export_path).unwrap(), original);

        let (_tmp_dest, dest_paths) = test_paths();
        let info = import_emblem(&dest_paths, &export_path, Some("Imported dragon")).unwrap();
        assert_eq!(info.slot, 1);
        assert_eq!(info.label, "Imported dragon");
        assert_eq!(
            std::fs::read(slot_path(&dest_paths, &info.group, 1)).unwrap(),
            original
        );

        let emblems = list_emblems(&dest_paths).unwrap();
        assert_eq!(emblems.len(), 1);
        assert_eq!(emblems[0].label, "Imported dragon");
    }

    #[test]
    fn import_rejects_files_that_are_too_small_to_be_an_emblem() {
        let (_tmp, paths) = test_paths();
        let bogus = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(bogus.path(), b"not an emblem").unwrap();

        let err = import_emblem(&paths, bogus.path(), None).unwrap_err();
        assert!(err.0.contains("doesn't look like a valid emblem"));
        assert!(list_groups(&paths).unwrap().is_empty());
    }

    #[test]
    fn import_without_a_label_leaves_it_blank() {
        let (_tmp, paths) = test_paths();
        let bytes = fake_captured_bytes();
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(f.path(), &bytes).unwrap();

        let info = import_emblem(&paths, f.path(), None).unwrap();
        assert_eq!(info.label, "");

        let info2 = import_emblem(&paths, f.path(), Some("   ")).unwrap();
        assert_eq!(info2.label, "", "whitespace-only label should be treated as blank");
    }

    #[test]
    fn duplicate_detection_ignores_differing_http_headers_but_matches_on_body() {
        let (_tmp, paths) = test_paths();
        let body = fake_captured_bytes();
        let mut framed_a = b"HTTP/1.1 200 OK\r\nDate: Mon, 01 Jan 2026 00:00:00 GMT\r\n\r\n".to_vec();
        framed_a.extend_from_slice(crate::emblem::format::strip_http(&body));
        let mut framed_b = b"HTTP/1.1 200 OK\r\nDate: Tue, 02 Jan 2026 00:00:00 GMT\r\n\r\n".to_vec();
        framed_b.extend_from_slice(crate::emblem::format::strip_http(&body));

        write_slot(&paths, "001", 1, &framed_a);
        write_slot(&paths, "002", 1, &framed_b);

        let mut different_body = crate::emblem::format::strip_http(&body).to_vec();
        different_body[0] = different_body[0].wrapping_add(1);
        let mut framed_c = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
        framed_c.extend_from_slice(&different_body);
        write_slot(&paths, "003", 1, &framed_c);

        let groups = find_duplicate_groups(&paths).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }

    #[test]
    fn no_duplicates_when_every_capture_is_unique() {
        let (_tmp, paths) = test_paths();
        write_slot(&paths, "001", 1, &fake_captured_bytes());
        let groups = find_duplicate_groups(&paths).unwrap();
        assert!(groups.is_empty());
    }
}
