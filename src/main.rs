use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let parse = |value: &std::ffi::OsString| {
        value
            .to_string_lossy()
            .parse::<i32>()
            .map_err(|_| "block coordinates must be 32-bit integers".to_string())
    };
    let result: Result<String, String> = match args.as_slice() {
        [command, root, map, output] if command == "snapshot" => {
            mapstrike::snapshot_project(Path::new(root), Path::new(map)).and_then(|snapshot| {
                mapstrike::write_new_snapshot(&snapshot, Path::new(output))?;
                Ok(format!("{}: {} source assets inventoried", snapshot.map_relative_path, snapshot.assets.len()))
            })
        }
        [command, name, output] if command == "new-blockout" => {
            mapstrike::blockout::Blockout::new(&name.to_string_lossy()).and_then(|document| {
                mapstrike::blockout::save_blockout(&document, Path::new(output), false)?;
                Ok(format!("new blockout: {}", Path::new(output).display()))
            })
        }
        [command, document, x0, y0, z0, x1, y1, z1] if command == "add-block" => (|| {
            let mut blockout = mapstrike::blockout::load_blockout(Path::new(document))?;
            let id = blockout.add_block([parse(x0)?,parse(y0)?,parse(z0)?],
                [parse(x1)?,parse(y1)?,parse(z1)?])?;
            mapstrike::blockout::save_blockout(&blockout, Path::new(document), true)?;
            Ok(format!("added block {id}"))
        })(),
        [command, document, id, dx, dy, dz] if command == "move-block" => (|| {
            let id = id.to_string_lossy().parse::<u32>().map_err(|_| "invalid block ID".to_string())?;
            let mut blockout = mapstrike::blockout::load_blockout(Path::new(document))?;
            blockout.move_block(id, [parse(dx)?,parse(dy)?,parse(dz)?])?;
            mapstrike::blockout::save_blockout(&blockout, Path::new(document), true)?;
            Ok(format!("moved block {id}"))
        })(),
        [command, document, output] if command == "export-obj" => {
            mapstrike::blockout::load_blockout(Path::new(document)).and_then(|blockout| {
                mapstrike::blockout::export_obj(&blockout, Path::new(output))?;
                Ok(format!("inspection OBJ exported: {}", Path::new(output).display()))
            })
        }
        _ => Err("usage: mapstrike snapshot <root> <map.vmap> <new-report.json> | new-blockout <name> <new-document.json> | add-block <document.json> <x0> <y0> <z0> <x1> <y1> <z1> | move-block <document.json> <id> <dx> <dy> <dz> | export-obj <document.json> <new.obj>".into()),
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
