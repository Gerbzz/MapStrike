//! Experimental CS2 VMAP source writer. Its KV2 field layout was checked against
//! a public CS2 `vmap 40` gameplay source, but Hammer acceptance is unverified.

use crate::blockout::{Block, Blockout, Spawn, Team};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

const MATERIAL: &str = "materials/dev/dev_measuregeneric01.vmat";
const FACES: [[usize; 4]; 6] = [
    [0, 3, 2, 1], // floor-facing
    [4, 5, 6, 7], // ceiling-facing
    [0, 1, 5, 4],
    [3, 7, 6, 2],
    [0, 4, 7, 3],
    [1, 2, 6, 5],
];
const NORMALS: [[i32; 3]; 6] = [
    [0, 0, -1],
    [0, 0, 1],
    [0, -1, 0],
    [0, 1, 0],
    [-1, 0, 0],
    [1, 0, 0],
];

struct Kv2 {
    text: String,
    depth: usize,
}

impl Kv2 {
    fn new() -> Self {
        Self {
            text: "<!-- dmx encoding keyvalues2 4 format vmap 40 -->\n".into(),
            depth: 0,
        }
    }

    fn line(&mut self, line: &str) {
        for _ in 0..self.depth {
            self.text.push('\t');
        }
        self.text.push_str(line);
        self.text.push('\n');
    }

    fn object(&mut self, name: &str, ty: Option<&str>) {
        self.line(&match ty {
            Some(ty) => format!("\"{name}\" \"{ty}\""),
            None => format!("\"{name}\""),
        });
        self.line("{");
        self.depth += 1;
    }

    fn end(&mut self, comma: bool) {
        self.depth -= 1;
        self.line(if comma { "}," } else { "}" });
    }

    fn field(&mut self, name: &str, ty: &str, value: &str) {
        self.line(&format!("\"{name}\" \"{ty}\" \"{value}\""));
    }

    fn array(&mut self, name: &str, ty: &str, values: &[String]) {
        self.line(&format!("\"{name}\" \"{ty}_array\""));
        self.line("[");
        self.depth += 1;
        for (index, value) in values.iter().enumerate() {
            self.line(&format!(
                "\"{value}\"{}",
                if index + 1 == values.len() { "" } else { "," }
            ));
        }
        self.depth -= 1;
        self.line("]");
    }

    fn empty(&mut self, name: &str, ty: &str) {
        self.array(name, ty, &[]);
    }

    fn id(&mut self, ids: &Ids, label: &str) {
        self.field("id", "elementid", &ids.uuid(label));
    }
}

struct Ids(Vec<u8>);

impl Ids {
    fn new(blockout: &Blockout) -> Result<Self, String> {
        serde_json::to_vec(blockout)
            .map(Self)
            .map_err(|error| error.to_string())
    }

    fn digest(&self, label: &str) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"MapStrike CS2 VMAP v1\0");
        hash.update(&self.0);
        hash.update([0]);
        hash.update(label.as_bytes());
        hash.finalize().into()
    }

    fn uuid(&self, label: &str) -> String {
        let mut bytes = self.digest(label);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let h = bytes[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        format!(
            "{}-{}-{}-{}-{}",
            &h[0..8],
            &h[8..12],
            &h[12..16],
            &h[16..20],
            &h[20..32]
        )
    }

    fn reference(&self, label: &str) -> String {
        let bytes = self.digest(label);
        let value = u64::from_le_bytes(bytes[..8].try_into().unwrap()) | 1;
        format!("0x{value:016x}")
    }
}

fn values<I, T>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: ToString,
{
    items.into_iter().map(|item| item.to_string()).collect()
}

fn half(value: i64) -> String {
    format!("{:.1}", value as f64 / 2.0)
}

fn relay_plugs(w: &mut Kv2, ids: &Ids, label: &str) {
    w.object("relayPlugData", Some("DmePlugList"));
    w.id(ids, label);
    for key in ["names", "dataTypes", "plugTypes", "descriptions"] {
        w.empty(
            key,
            if key == "dataTypes" || key == "plugTypes" {
                "int"
            } else {
                "string"
            },
        );
    }
    w.end(false);
}

fn transform(w: &mut Kv2, ids: &Ids, label: &str, origin: &str, yaw: u16) {
    w.field("origin", "vector3", origin);
    w.field("angles", "qangle", &format!("0 {yaw} 0"));
    w.field("scales", "vector3", "1 1 1");
    w.field("transformLocked", "bool", "0");
    w.object("transformPin", Some("DmElement"));
    w.id(ids, &format!("{label}.pin"));
    w.field("name", "string", "transformPin");
    w.field("referenceName", "string", "");
    w.field("targetReferenceID", "uint64", "0x0");
    w.field("offsetOrigin", "vector3", "0 0 0");
    w.field("offsetAngles", "qangle", "0 0 0");
    w.field("pinAngles", "bool", "1");
    w.field("twoWay", "bool", "0");
    w.end(false);
    w.field("force_hidden", "bool", "0");
    w.field("editorOnly", "bool", "0");
    w.field("customVisGroup", "string", "");
    w.field("randomSeed", "int", "1");
}

fn stream(
    w: &mut Kv2,
    ids: &Ids,
    label: &str,
    attr: &str,
    ty: &str,
    flags: u8,
    data: &[String],
    comma: bool,
) {
    w.object("CDmePolygonMeshDataStream", None);
    w.id(ids, label);
    w.field("name", "string", &format!("{attr}:0"));
    w.field("standardAttributeName", "string", attr);
    w.field("semanticName", "string", attr);
    w.field("semanticIndex", "int", "0");
    w.field("vertexBufferLocation", "int", "0");
    w.field("dataStateFlags", "int", &flags.to_string());
    w.field("subdivisionBinding", "element", "");
    w.array("data", ty, data);
    w.end(comma);
}

fn data_array(
    w: &mut Kv2,
    ids: &Ids,
    label: &str,
    name: &str,
    size: usize,
    streams: &[(&str, &str, u8, Vec<String>)],
) {
    w.object(name, Some("CDmePolygonMeshDataArray"));
    w.id(ids, label);
    w.field("size", "int", &size.to_string());
    w.line("\"streams\" \"element_array\"");
    w.line("[");
    w.depth += 1;
    for (i, (attr, ty, flags, entries)) in streams.iter().enumerate() {
        stream(
            w,
            ids,
            &format!("{label}.{attr}"),
            attr,
            ty,
            *flags,
            entries,
            i + 1 != streams.len(),
        );
    }
    w.depth -= 1;
    w.line("]");
    w.end(false);
}

fn mesh(w: &mut Kv2, ids: &Ids, block: &Block, node: usize, comma: bool) {
    let label = format!("mesh.{}", block.id);
    let lengths: [i64; 3] =
        std::array::from_fn(|axis| i64::from(block.max[axis]) - i64::from(block.min[axis]));
    let positions: [[i64; 3]; 8] = [
        [-lengths[0], -lengths[1], -lengths[2]],
        [lengths[0], -lengths[1], -lengths[2]],
        [lengths[0], lengths[1], -lengths[2]],
        [-lengths[0], lengths[1], -lengths[2]],
        [-lengths[0], -lengths[1], lengths[2]],
        [lengths[0], -lengths[1], lengths[2]],
        [lengths[0], lengths[1], lengths[2]],
        [-lengths[0], lengths[1], lengths[2]],
    ];
    let mut vertex_edge = [0usize; 8];
    let mut seen_vertex = [false; 8];
    let mut edges = Vec::with_capacity(24);
    let mut next = Vec::with_capacity(24);
    let mut face_of_edge = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut tangents = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    for (face, quad) in FACES.iter().enumerate() {
        let first = edges.len();
        let u = (0..3)
            .map(|axis| (positions[quad[1]][axis] - positions[quad[0]][axis]).abs())
            .max()
            .unwrap() as f64
            / 128.0;
        let v = (0..3)
            .map(|axis| (positions[quad[2]][axis] - positions[quad[1]][axis]).abs())
            .max()
            .unwrap() as f64
            / 128.0;
        for corner in 0..4 {
            let from = quad[corner];
            let to = quad[(corner + 1) % 4];
            if !seen_vertex[from] {
                vertex_edge[from] = edges.len();
                seen_vertex[from] = true;
            }
            edges.push((from, to));
            next.push(first + (corner + 1) % 4);
            face_of_edge.push(face);
            let normal = NORMALS[face];
            normals.push(format!("{} {} {}", normal[0], normal[1], normal[2]));
            let tangent = if face >= 4 { "0 1 0 1" } else { "1 0 0 1" };
            tangents.push(tangent.into());
            let (s, t) = match corner {
                0 => (0.0, 0.0),
                1 => (u, 0.0),
                2 => (u, v),
                _ => (0.0, v),
            };
            uvs.push(format!("{s:.6} {t:.6}"));
        }
    }
    let mut opposite = Vec::with_capacity(24);
    let mut edge_data = Vec::with_capacity(24);
    let mut pair_ids = std::collections::HashMap::new();
    for &(from, to) in &edges {
        let other = edges
            .iter()
            .position(|&(a, b)| a == to && b == from)
            .unwrap();
        opposite.push(other);
        let key = (from.min(to), from.max(to));
        let count = pair_ids.len();
        edge_data.push(*pair_ids.entry(key).or_insert(count));
    }
    w.object("CMapMesh", None);
    w.id(ids, &label);
    w.field("nodeID", "int", &node.to_string());
    w.field("referenceID", "uint64", &ids.reference(&label));
    w.empty("children", "element");
    w.empty("variableTargetKeys", "string");
    w.empty("variableNames", "string");
    w.object("meshData", Some("CDmePolygonMesh"));
    w.id(ids, &format!("{label}.data"));
    w.field("name", "string", "meshData");
    let arrays = [
        ("vertexEdgeIndices", values(vertex_edge)),
        ("vertexDataIndices", values(0..8)),
        (
            "edgeVertexIndices",
            values(edges.iter().map(|&(from, _)| from)),
        ),
        ("edgeOppositeIndices", values(opposite)),
        ("edgeNextIndices", values(next)),
        ("edgeFaceIndices", values(face_of_edge)),
        ("edgeDataIndices", values(edge_data)),
        ("edgeVertexDataIndices", values(0..24)),
        ("faceEdgeIndices", values((0..6).map(|f| f * 4))),
        ("faceDataIndices", values(0..6)),
    ];
    for (name, items) in arrays {
        w.array(name, "int", &items);
    }
    w.array("materials", "string", &[MATERIAL.into()]);
    let vertex_positions = positions
        .iter()
        .map(|p| format!("{} {} {}", half(p[0]), half(p[1]), half(p[2])))
        .collect();
    data_array(
        w,
        ids,
        &format!("{label}.vertices"),
        "vertexData",
        8,
        &[("position", "vector3", 3, vertex_positions)],
    );
    data_array(
        w,
        ids,
        &format!("{label}.corners"),
        "faceVertexData",
        24,
        &[
            ("texcoord", "vector2", 1, uvs),
            ("normal", "vector3", 1, normals),
            ("tangent", "vector4", 1, tangents),
        ],
    );
    data_array(
        w,
        ids,
        &format!("{label}.edges"),
        "edgeData",
        12,
        &[("flags", "int", 3, values([0; 12]))],
    );
    let face_tangents = (0..6)
        .map(|face| if face >= 4 { "0 1 0 0" } else { "1 0 0 0" }.to_string())
        .collect();
    let face_bitangents = (0..6)
        .map(|face| if face <= 1 { "0 1 0 0" } else { "0 0 -1 0" }.to_string())
        .collect();
    data_array(
        w,
        ids,
        &format!("{label}.faces"),
        "faceData",
        6,
        &[
            ("textureScale", "vector2", 0, values(["0.25 0.25"; 6])),
            ("textureAxisU", "vector4", 0, face_tangents),
            ("textureAxisV", "vector4", 0, face_bitangents),
            ("materialindex", "int", 8, values([0; 6])),
            ("flags", "int", 3, values([0; 6])),
            ("lightmapScaleBias", "int", 1, values([0; 6])),
        ],
    );
    w.object("subdivisionData", Some("CDmePolygonMeshSubdivisionData"));
    w.id(ids, &format!("{label}.subdivision"));
    w.array("subdivisionLevels", "int", &values([0; 24]));
    w.empty("streams", "element");
    w.end(false);
    w.end(false); // meshData
    let center = (0..3)
        .map(|axis| half(i64::from(block.min[axis]) + i64::from(block.max[axis])))
        .collect::<Vec<_>>()
        .join(" ");
    transform(w, ids, &label, &center, 0);
    w.field("disableShadows", "int", "0");
    w.field("bakelighting", "bool", "1");
    w.field("physicsType", "string", "default");
    w.field("physicsSimplificationOverride", "bool", "0");
    w.end(comma);
}

fn spawn(w: &mut Kv2, ids: &Ids, entity: &Spawn, node: usize, comma: bool) {
    let label = format!("spawn.{}", entity.id);
    let classname = match entity.team {
        Team::Terrorists => "info_player_terrorist",
        Team::CounterTerrorists => "info_player_counterterrorist",
    };
    w.object("CMapEntity", None);
    w.id(ids, &label);
    w.field("nodeID", "int", &node.to_string());
    w.field("referenceID", "uint64", &ids.reference(&label));
    w.empty("children", "element");
    w.empty("variableTargetKeys", "string");
    w.empty("variableNames", "string");
    relay_plugs(w, ids, &format!("{label}.plugs"));
    w.empty("connectionsData", "element");
    w.object("entity_properties", Some("EditGameClassProps"));
    w.id(ids, &format!("{label}.properties"));
    w.field("classname", "string", classname);
    w.field("targetname", "string", "");
    w.field("vscripts", "string", "");
    w.field("priority", "string", "0");
    w.field("enabled", "string", "1");
    w.end(false);
    w.field("hitNormal", "vector3", "0 0 1");
    w.field("isProceduralEntity", "bool", "0");
    let origin = entity
        .origin
        .map(|coordinate| coordinate.to_string())
        .join(" ");
    transform(w, ids, &label, &origin, entity.yaw);
    w.end(comma);
}

fn render(blockout: &Blockout) -> Result<String, String> {
    blockout.validate()?;
    if blockout.blocks.is_empty() || blockout.blocks.len() > 128 || blockout.spawns.len() > 64 {
        return Err("VMAP proof requires 1-128 blocks and at most 64 spawns".into());
    }
    let t = blockout
        .spawns
        .iter()
        .filter(|s| s.team == Team::Terrorists)
        .count();
    let ct = blockout
        .spawns
        .iter()
        .filter(|s| s.team == Team::CounterTerrorists)
        .count();
    if t == 0 || ct == 0 {
        return Err("VMAP proof requires at least one T and one CT spawn".into());
    }
    for spawn in &blockout.spawns {
        if blockout.blocks.iter().any(|block| {
            (0..3).all(|axis| {
                block.min[axis] < spawn.origin[axis] && spawn.origin[axis] < block.max[axis]
            })
        }) {
            return Err(format!("spawn {} lies inside a solid block", spawn.id));
        }
    }
    let ids = Ids::new(blockout)?;
    let mut w = Kv2::new();
    w.object("$prefix_element$", None);
    w.id(&ids, "prefix");
    w.array("map_asset_references", "string", &[MATERIAL.into()]);
    w.array(
        "map_subasset_reference_types",
        "string",
        &values(["entity", "entity"]),
    );
    w.array(
        "map_subasset_reference_names",
        "string",
        &values(["info_player_terrorist", "info_player_counterterrorist"]),
    );
    w.array("map_subasset_reference_counts", "int", &values([t, ct]));
    w.end(false);
    w.object("CMapRootElement", None);
    w.id(&ids, "root");
    w.field("isprefab", "bool", "0");
    w.field("editorbuild", "int", "10847");
    w.field("editorversion", "int", "400");
    w.field("itemFile", "string", "");
    w.object("defaultcamera", Some("CStoredCamera"));
    w.id(&ids, "camera");
    w.field("position", "vector3", "0 -1000 1000");
    w.field("lookat", "vector3", "0 0 0");
    w.end(false);
    w.object("3dcameras", Some("CStoredCameras"));
    w.id(&ids, "cameras");
    w.field("activecamera", "int", "-1");
    w.empty("cameras", "element");
    w.end(false);
    w.object("world", Some("CMapWorld"));
    w.id(&ids, "world");
    w.field("nodeID", "int", "1");
    w.field("referenceID", "uint64", "0x0");
    w.line("\"children\" \"element_array\"");
    w.line("[");
    w.depth += 1;
    let children = blockout.blocks.len() + blockout.spawns.len();
    for (i, block) in blockout.blocks.iter().enumerate() {
        mesh(&mut w, &ids, block, i + 2, i + 1 < children);
    }
    for (i, entity) in blockout.spawns.iter().enumerate() {
        spawn(
            &mut w,
            &ids,
            entity,
            blockout.blocks.len() + i + 2,
            blockout.blocks.len() + i + 1 < children,
        );
    }
    w.depth -= 1;
    w.line("]");
    w.empty("variableTargetKeys", "string");
    w.empty("variableNames", "string");
    relay_plugs(&mut w, &ids, "world.plugs");
    w.empty("connectionsData", "element");
    w.object("entity_properties", Some("EditGameClassProps"));
    w.id(&ids, "world.properties");
    w.field("classname", "string", "worldspawn");
    w.field("targetname", "string", "");
    w.field("skyname", "string", "sky_day01_01");
    w.end(false);
    w.field("nextDecalID", "int", "0");
    w.field("fixupEntityNames", "bool", "1");
    w.field("mapUsageType", "string", "standard");
    transform(&mut w, &ids, "world", "0 0 0", 0);
    w.end(false);
    w.object("visbility", Some("CVisibilityMgr")); // Valve's serialized key spelling
    w.id(&ids, "visibility");
    w.field("nodeID", "int", "0");
    w.field("referenceID", "uint64", "0x0");
    w.empty("children", "element");
    w.empty("variableTargetKeys", "string");
    w.empty("variableNames", "string");
    w.empty("nodes", "element");
    w.empty("hiddenFlags", "int");
    transform(&mut w, &ids, "visibility", "0 0 0", 0);
    w.end(false);
    w.object("mapVariables", Some("CMapVariableSet"));
    w.id(&ids, "variables");
    for name in [
        "variableNames",
        "variableValues",
        "variableTypeNames",
        "variableTypeParameters",
        "variableGroupNames",
    ] {
        w.empty(name, "string");
    }
    w.empty("m_ChoiceGroups", "element");
    w.empty("variableAndChoiceOrder", "int");
    w.end(false);
    w.object("rootSelectionSet", Some("CMapSelectionSet"));
    w.id(&ids, "selection");
    w.empty("children", "element");
    w.field("selectionSetName", "string", "");
    w.field("selectionSetData", "element", "");
    w.end(false);
    w.empty("m_ReferencedMeshSnapshots", "element");
    w.field("m_bIsCordoning", "bool", "0");
    w.field("m_bCordonsVisible", "bool", "0");
    w.empty("nodeInstanceData", "element");
    w.end(false);
    Ok(w.text)
}

/// Write a new VMAP source. Never overwrites user-authored Hammer maps.
pub fn export_vmap(blockout: &Blockout, output: &Path) -> Result<(), String> {
    if !output
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("vmap"))
    {
        return Err("choose a new .vmap output path".into());
    }
    let text = render(blockout)?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    pending
        .write_all(text.as_bytes())
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    pending
        .persist_noclobber(output)
        .map_err(|e| e.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FACES, export_vmap, render};
    use crate::blockout::{Blockout, Team};

    fn room() -> Blockout {
        let mut room = Blockout::new("Test room").unwrap();
        room.add_room([-512, -512, 0], [1024, 1024, 256], 16)
            .unwrap();
        room.add_spawn(Team::Terrorists, [-256, 0, 16], 0).unwrap();
        room.add_spawn(Team::CounterTerrorists, [256, 0, 16], 180)
            .unwrap();
        room
    }

    #[test]
    fn each_box_edge_has_one_reverse_and_unique_face() {
        let edges = FACES
            .iter()
            .flat_map(|face| (0..4).map(move |i| (face[i], face[(i + 1) % 4])))
            .collect::<Vec<_>>();
        assert_eq!(edges.len(), 24);
        for &(from, to) in &edges {
            assert_eq!(
                edges.iter().filter(|&&(a, b)| a == to && b == from).count(),
                1
            );
        }
    }

    #[test]
    fn emits_room_meshes_and_actual_team_entity_classnames() {
        let text = render(&room()).unwrap();
        assert!(text.starts_with("<!-- dmx encoding keyvalues2 4 format vmap 40 -->"));
        assert_eq!(text.matches("\"CMapMesh\"\n").count(), 5);
        assert_eq!(text.matches("\"CMapEntity\"\n").count(), 2);
        assert!(text.contains("\"classname\" \"string\" \"info_player_terrorist\""));
        assert!(text.contains("\"classname\" \"string\" \"info_player_counterterrorist\""));
        assert!(text.contains("\"origin\" \"vector3\" \"-256 0 16\""));
        assert!(text.contains("\"angles\" \"qangle\" \"0 180 0\""));
        assert_eq!(text, render(&room()).unwrap());
    }

    #[test]
    fn refuses_existing_map_and_missing_team() {
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("room.vmap");
        let room = room();
        export_vmap(&room, &output).unwrap();
        let first = std::fs::read(&output).unwrap();
        assert!(export_vmap(&room, &output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), first);
        let mut no_ct = room;
        no_ct.spawns.retain(|spawn| spawn.team == Team::Terrorists);
        assert!(render(&no_ct).is_err());
        let wrong_extension = dir.path().join("not-a-map.txt");
        assert!(export_vmap(&no_ct, &wrong_extension).is_err());
        assert!(!wrong_extension.exists());
    }

    #[test]
    fn refuses_spawn_inside_solid_without_creating_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("bad.vmap");
        let mut room = room();
        room.spawns[0].origin = [0, 0, -8];
        assert!(export_vmap(&room, &output).is_err());
        assert!(!output.exists());
    }
}
