use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub id: u32,
    pub min: [i32; 3],
    pub max: [i32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blockout {
    pub schema_version: u32,
    pub name: String,
    pub blocks: Vec<Block>,
}

impl Blockout {
    pub fn new(name: &str) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("blockout name is required".into());
        }
        Ok(Self {
            schema_version: 1,
            name: name.into(),
            blocks: Vec::new(),
        })
    }

    pub fn add_block(&mut self, min: [i32; 3], max: [i32; 3]) -> Result<u32, String> {
        if (0..3).any(|axis| max[axis] <= min[axis]) {
            return Err("each block needs positive width, depth and height".into());
        }
        let id = self
            .blocks
            .iter()
            .map(|block| block.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| "block ID overflow".to_string())?;
        self.blocks.push(Block { id, min, max });
        Ok(id)
    }

    pub fn move_block(&mut self, id: u32, delta: [i32; 3]) -> Result<(), String> {
        let block = self
            .blocks
            .iter_mut()
            .find(|block| block.id == id)
            .ok_or_else(|| format!("block {id} not found"))?;
        let mut min = block.min;
        let mut max = block.max;
        for axis in 0..3 {
            min[axis] = min[axis]
                .checked_add(delta[axis])
                .ok_or("block position overflow")?;
            max[axis] = max[axis]
                .checked_add(delta[axis])
                .ok_or("block position overflow")?;
        }
        block.min = min;
        block.max = max;
        Ok(())
    }
}

pub fn load_blockout(path: &Path) -> Result<Blockout, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let blockout: Blockout = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if blockout.schema_version != 1 {
        return Err("unsupported blockout document version".into());
    }
    let mut ids = std::collections::HashSet::new();
    for block in &blockout.blocks {
        if !ids.insert(block.id) || (0..3).any(|axis| block.max[axis] <= block.min[axis]) {
            return Err("invalid blockout geometry or repeated ID".into());
        }
    }
    Ok(blockout)
}

pub fn save_blockout(blockout: &Blockout, path: &Path, overwrite: bool) -> Result<(), String> {
    if overwrite && path.exists() {
        let existing = load_blockout(path)?;
        if existing.name != blockout.name {
            return Err("destination belongs to a different blockout".into());
        }
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(blockout).map_err(|error| error.to_string())?;
    pending
        .write_all(&bytes)
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    if overwrite {
        pending
            .persist(path)
            .map_err(|error| error.error.to_string())?;
    } else {
        pending
            .persist_noclobber(path)
            .map_err(|error| error.error.to_string())?;
    }
    Ok(())
}

/// Output inspection geometry only. OBJ is not a Valve .vmap build artifact.
pub fn export_obj(blockout: &Blockout, output: &Path) -> Result<(), String> {
    if blockout.blocks.is_empty() {
        return Err("add a block before export".into());
    }
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    writeln!(
        pending,
        "# MapStrike blockout inspection mesh: {}",
        blockout.name
    )
    .map_err(|error| error.to_string())?;
    let sides = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [3, 7, 6, 2],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for (index, block) in blockout.blocks.iter().enumerate() {
        let [x0, y0, z0] = block.min;
        let [x1, y1, z1] = block.max;
        let corners = [
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y1, z0],
            [x0, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x1, y1, z1],
            [x0, y1, z1],
        ];
        writeln!(pending, "o block_{}", block.id).map_err(|error| error.to_string())?;
        for point in corners {
            writeln!(pending, "v {} {} {}", point[0], point[1], point[2])
                .map_err(|error| error.to_string())?;
        }
        let first = index * 8 + 1;
        for side in sides {
            let [a, b, c, d] = side.map(|corner| corner + first);
            writeln!(pending, "f {a} {b} {c}").map_err(|error| error.to_string())?;
            writeln!(pending, "f {a} {c} {d}").map_err(|error| error.to_string())?;
        }
    }
    pending
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    pending
        .persist_noclobber(output)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Blockout, export_obj, load_blockout, save_blockout};

    #[test]
    fn creates_moves_saves_and_exports_valid_block_geometry() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut blockout = Blockout::new("Test room").unwrap();
        let id = blockout.add_block([0, 0, 0], [128, 128, 64]).unwrap();
        blockout.move_block(id, [16, -8, 4]).unwrap();
        let document = dir.path().join("room.mapstrike.json");
        save_blockout(&blockout, &document, false).unwrap();
        let reopened = load_blockout(&document).unwrap();
        assert_eq!(reopened.blocks[0].min, [16, -8, 4]);
        let mesh = dir.path().join("room.obj");
        export_obj(&reopened, &mesh).unwrap();
        let obj = std::fs::read_to_string(&mesh).unwrap();
        assert_eq!(obj.lines().filter(|line| line.starts_with("v " )).count(), 8);
        assert_eq!(obj.lines().filter(|line| line.starts_with("f " )).count(), 12);
        assert!(obj.lines().any(|line| line == "v 144 120 68"));
        assert!(export_obj(&reopened, &mesh).is_err());
    }

    #[test]
    fn rejects_zero_volume_and_overflow_without_mutating_block() {
        let mut blockout = Blockout::new("Test").unwrap();
        assert!(blockout.add_block([0, 0, 0], [0, 1, 1]).is_err());
        let id = blockout
            .add_block([i32::MAX - 2, 0, 0], [i32::MAX, 1, 1])
            .unwrap();
        assert!(blockout.move_block(id, [10, 0, 0]).is_err());
        assert_eq!(blockout.blocks[0].min[0], i32::MAX - 2);
    }

    #[test]
    fn refuses_to_overwrite_unrelated_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("existing.json");
        std::fs::write(&path, b"{\"unrelated\":true}").unwrap();
        let blockout = Blockout::new("Test").unwrap();
        assert!(save_blockout(&blockout, &path, true).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"unrelated\":true}");
    }
}
