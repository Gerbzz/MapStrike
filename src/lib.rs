use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;

pub mod blockout;

#[cfg(feature = "native-ui")]
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceAsset {
    pub relative_path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapSnapshot {
    pub schema_version: u32,
    pub map_relative_path: String,
    pub assets: Vec<SourceAsset>,
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn relative(root: &Path, path: &Path) -> Result<String, String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|error| error.to_string())?
        .to_string_lossy()
        .replace('\\', "/"))
}

/// Snapshot editable map inputs. This does not parse or build a Valve map.
pub fn snapshot_project(root: &Path, map_file: &Path) -> Result<MapSnapshot, String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    if !root.is_dir() {
        return Err("map project root must be a directory".into());
    }
    let map = map_file.canonicalize().map_err(|error| error.to_string())?;
    if !map.starts_with(&root)
        || !map.is_file()
        || !map
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vmap"))
    {
        return Err("choose a .vmap file inside the project root".into());
    }
    if map.metadata().map_err(|error| error.to_string())?.len() == 0 {
        return Err("map source is empty".into());
    }
    let allowed = ["vmap", "vmat", "vmdl", "fbx", "obj", "tga", "png"];
    let mut assets = Vec::new();
    for entry in WalkDir::new(&root).follow_links(false) {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_type().is_symlink() {
            return Err(format!(
                "linked files require explicit review: {}",
                entry.path().display()
            ));
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let extension = entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        if !allowed
            .iter()
            .any(|item| extension.eq_ignore_ascii_case(item))
        {
            continue;
        }
        let path = entry
            .path()
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !path.starts_with(&root) {
            return Err("asset escapes map project root".into());
        }
        assets.push(SourceAsset {
            relative_path: relative(&root, &path)?,
            bytes: path.metadata().map_err(|error| error.to_string())?.len(),
            sha256: hash_file(&path)?,
        });
    }
    assets.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(MapSnapshot {
        schema_version: 1,
        map_relative_path: relative(&root, &map)?,
        assets,
    })
}

pub fn write_new_snapshot(snapshot: &MapSnapshot, output: &Path) -> Result<(), String> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(snapshot).map_err(|error| error.to_string())?;
    pending
        .write_all(&bytes)
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    pending
        .persist_noclobber(output)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MapSnapshot, snapshot_project, write_new_snapshot};
    use std::fs;

    #[test]
    fn inventories_map_sources_deterministically_and_reopens_snapshot() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path().join("project");
        fs::create_dir_all(root.join("maps")).unwrap();
        fs::create_dir_all(root.join("materials")).unwrap();
        let map = root.join("maps/test.vmap");
        fs::write(&map, b"map source").unwrap();
        fs::write(root.join("materials/wall.vmat"), b"material source").unwrap();
        fs::write(root.join("notes.txt"), b"not an asset").unwrap();
        let snapshot = snapshot_project(&root, &map).unwrap();
        assert_eq!(snapshot.map_relative_path, "maps/test.vmap");
        assert_eq!(snapshot.assets.len(), 2);
        assert_eq!(snapshot.assets[0].relative_path, "maps/test.vmap");
        assert_eq!(snapshot.assets[1].relative_path, "materials/wall.vmat");
        let output = dir.path().join("snapshot.json");
        write_new_snapshot(&snapshot, &output).unwrap();
        let reopened: MapSnapshot = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert_eq!(reopened, snapshot);
        assert!(write_new_snapshot(&snapshot, &output).is_err());
    }

    #[test]
    fn rejects_map_outside_project_root() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path().join("project");
        fs::create_dir(&root).unwrap();
        let outside = dir.path().join("outside.vmap");
        fs::write(&outside, b"map").unwrap();
        assert!(snapshot_project(&root, &outside).is_err());
    }
}
