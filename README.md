<p align="center">
  <img src="assets/branding/MapStrike.png" alt="Original MapStrike turtle logo" width="160" height="160">
</p>

<h1 align="center">MapStrike</h1>

<p align="center">
  <a href="https://prowebdevsolutions.com"><picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/PWDS-logo-orange.svg">
    <img src="assets/branding/PWDS-logo.png" alt="PWDS company logo" width="120">
  </picture></a>
</p>

**Lay out the space before the details.**

MapStrike gives early map ideas a simple geometric form. Place and move boxes in a named blockout, save the layout, and export an OBJ to inspect elsewhere; a separate tool can inventory an existing map folder. This draft also writes an experimental CS2 `.vmap` source for rooms with both team spawns.

## Native renderer preview

![Actual MapStrike native blockout form showing a synthetic four-box Courtyard Study](docs/screenshots/mapstrike-blockout-prototype.png)

This 650 x 370 image is a direct egui capture of the current form-based blockout UI with a [synthetic saved document](docs/screenshots/fixtures/courtyard-study.json). The evidence-only capture helper is in [draft PR #1](https://github.com/Gerbzz/MapStrike/pull/1); the application UI source is unchanged from `main`. This is a renderer view, not a record of pointer use or a 3D map preview. [Capture provenance and limits](docs/screenshots/README.md) identify the exact source, build, and fixture.

The screenshot shows the published `main` UI before the room, team-spawn, and experimental VMAP controls in this draft branch. The draft uses the approved MapStrike logo on its startup view and leaves the editor header for navigation. A new capture and visual inspection are pending.

## Highlights

- Build and revise box-based layouts.
- Generate an open room from a floor and four walls, then place T/CT spawn markers.
- Save a versioned blockout document.
- Export OBJ geometry or inventory source files.
- Write a new, experimental CS2 VMAP source with box meshes and T/CT spawn entities for Hammer validation.

## Current status

Early source prototype. The room, spawn, and experimental VMAP model passed [11 core and 13 native-model tests plus CLI and window builds on hosted Windows](https://github.com/Gerbzz/MapStrike/actions/runs/38709554651). Live UI inspection, Hammer open and compile, and CS2 walking checks remain open; this is not a playable CS2 map claim.

## Two focused workflows

### Build a box layout

Create a named blockout, add axis-aligned boxes with integer minimum and maximum coordinates, and move a box by its ID. A room command adds five boxes around a specified interior. Spawn markers store team, position, and yaw; they are project data and are not included in the OBJ mesh. Spawn collision and player clearance are not checked yet. The document validates positive width, depth, and height, and protects against coordinate or ID overflow. Save and reopen the versioned JSON document; export its boxes as triangle geometry in a new OBJ file. Version 1 documents reopen as version 2 with no spawns.

The desktop UI exposes this layout workflow. The OBJ is intended for inspection or further experimentation. A separate `export-vmap` action writes new KeyValues2 `vmap 40` source with six-sided box meshes and T/CT spawn entities. It refuses an existing path and requires at least one spawn for each team. Its source structure is based on a [public CS2 gameplay map](https://github.com/ryanpolasky/CS2-Uno/blob/5eec328523aca493653cf6849df62b0c783531f2/maps/prefabs/uno_by_imania/uno_by_imania_gameplay.vmap); no map content was copied. [Format evidence, foundation comparison, and acceptance checklist](docs/VMAP_PROOF.md) state the validation limits.

### Inventory an existing map folder

Point the CLI at a project root and a <code>.vmap</code> file inside it. MapStrike records the relative map path and hashes recognized source assets into a JSON snapshot. It does **not** parse VMAP geometry or modify any source asset. The inventory rejects linked files that require separate review.

| File | Role |
| --- | --- |
| Blockout <code>.json</code> | Editable name, box coordinates, and team spawn markers, schema version 2 |
| Export <code>.obj</code> | New inspection mesh generated from the blockout |
| Snapshot <code>.json</code> | Relative paths, byte lengths, and SHA-256 hashes of selected source files |
| Source <code>.vmap</code> | Existing input to inventory, or a new experimental export; never treated as an editable MapStrike document |

## Data checks behind the workflow

A blockout is a small schema-versioned model, not a text export from Hammer. Each box has a stable numeric ID and opposing minimum/maximum corners. Moving a box checks the resulting coordinates before saving, and JSON writes are staged so an error does not silently replace the previous document.

The inventory accepts a map file within the selected root and recognizes <code>.vmap</code>, <code>.vmat</code>, <code>.vmdl</code>, <code>.fbx</code>, <code>.obj</code>, <code>.tga</code>, and <code>.png</code> source assets. Its report is an audit of paths, sizes, and hashes; it makes no claim about whether those assets will compile or load in the game.

## Build and run

Use Windows and a Rust toolchain that supports edition 2024. The native window uses Rust egui/eframe components.

~~~powershell
cargo test --locked --features native-ui
cargo build --locked --features native-ui --bins
cargo run --locked --features native-ui --bin mapstrike-ui
~~~

The CLI exposes both workflows. Choose new paths for the JSON document and OBJ output:

~~~powershell
cargo run --locked --bin mapstrike -- new-blockout <name> <new-document.json>
cargo run --locked --bin mapstrike -- add-block <document.json> <x0> <y0> <z0> <x1> <y1> <z1>
cargo run --locked --bin mapstrike -- add-room <document.json> <x> <y> <z> <width> <depth> <height> <thickness>
cargo run --locked --bin mapstrike -- add-spawn <document.json> <t|ct> <x> <y> <z> <yaw>
cargo run --locked --bin mapstrike -- move-block <document.json> <id> <dx> <dy> <dz>
cargo run --locked --bin mapstrike -- export-obj <document.json> <new.obj>
cargo run --locked --bin mapstrike -- export-vmap <document.json> <new.vmap>
cargo run --locked --bin mapstrike -- snapshot <project-root> <map.vmap> <new-report.json>
~~~

The edit commands update the blockout document; both exports create separate files and refuse an existing destination. The snapshot is read-only with respect to the source folder. A VMAP export is a candidate Hammer source, not a compile or play result.

## What is still open

The geometry vocabulary is intentionally limited to boxes. MapStrike does not yet edit existing VMAP content, compile a playable level, validate materials and player clearance in Hammer, generate lighting or navigation, or round-trip through Valve tools. Visual workflow review and clean-machine packaging have not passed acceptance.

A broader brush/mesh editor, richer project organization, real map-format interchange, and in-game validation are possible directions. They are not part of this published prototype.

This repository grants no public license for its project source. Third-party crates retain their own licenses.

## PWDS brand artwork

The company mark in this masthead is the existing [official PWDS logo](https://github.com/Gerbzz/PWDS-Website/blob/main/public/prowebdevsolutions-logo.png), with the [official orange display asset](https://prowebdevsolutions.com/images/prowebdevsolutions-logo-orange.svg) for dark backgrounds. Any app logo above it remains the original Strike suite artwork.
