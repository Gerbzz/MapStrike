# Experimental CS2 VMAP proof

MapStrike's draft `export-vmap` command writes a **new** KeyValues2 `vmap 40` source file from its own blockout JSON. It creates a `CMapMesh` for each box and a `CMapEntity` for every T/CT marker. It does not parse or modify existing VMAP files. Output uses a stock development material path and never packages Valve or third-party assets. A pre-existing output path is left untouched.

The format fields and team entity names were checked against the [public CS2 Uno gameplay source at commit `5eec328`](https://github.com/ryanpolasky/CS2-Uno/blob/5eec328523aca493653cf6849df62b0c783531f2/maps/prefabs/uno_by_imania/uno_by_imania_gameplay.vmap). That repository has no declared reuse license, so no authored map content is incorporated. MapStrike generates its own box topology and values. The development material path is listed in the [CS2 file index](https://github.com/SteamTracking/GameTracking-CS2/blob/master/game/core/pak01_dir.txt). Valve's Hammer remains the only relevant acceptance tool for opening and compiling the result.

## Foundation choice

| Candidate | Workflow and code quality evidence | Fit |
| --- | --- | --- |
| [TrenchBroom](https://github.com/TrenchBroom/TrenchBroom) | Mature C++/Qt editor with 3D and orthographic views, brush/entity tools, undo, backups, tests and CI; GPL-3.0. | Strong interaction reference, but its Quake `.map` formats are not a CS2 `.vmap` path. Do not copy GPL code into this repository. |
| [Source 2 Viewer / ValveResourceFormat](https://github.com/ValveResourceFormat/ValveResourceFormat) | Active C# MIT parser, viewer, renderer and decompiler with CLI/GUI and tests. | Useful independent Source 2 inspection reference. It is not a Rust CS2 Hammer editor or proof that new maps compile. No code or assets are imported. |
| [bevy_trenchbroom](https://github.com/Noxmore/bevy_trenchbroom) | Rust, MIT/Apache-2.0, with Bevy loading of TrenchBroom `.map` and Quake `.bsp`; maintainer describes low-maintenance status. | Rust fit, but its Quake pipeline cannot establish CS2 VMAP support. |

**Decision:** retain MapStrike's small Rust blockout model and use actual CS2 VMAP sources as field-level references. Adopt interaction ideas such as visual editing and undo later only after the export path is proven in Hammer. Preserve the user's existing project and branding; do not replace it with a different engine's map workflow.

## Acceptance checklist

| Slice | State |
| --- | --- |
| Create room, add T/CT markers, save, reopen, reject invalid dimensions/yaw | Implemented; 8 core tests passed in [hosted Windows run 38707854215](https://github.com/Gerbzz/MapStrike/actions/runs/38707854215). |
| Write a new VMAP, keep existing destinations intact, reject missing team, repeat export safely | Implemented in draft; hosted test and native UI run pending. |
| Open exported VMAP in CS2 Hammer and inspect box geometry, material, and spawn placement | Unverified; Workshop Tools are not installed on this PC. |
| Compile, load in CS2, and walk between both team spawns | Missing. No playable-map claim. |
| Player clearance, lighting, navigation, visual manipulation, undo/redo, existing VMAP editing | Missing. |

The installed CS2 game has `cs2.exe`, but this PC has no `hammer.exe`, `resourcecompiler.exe`, or editable `content/csgo` directory. This draft does not install Workshop Tools or run a large local build. A subsequent validation must use an authorized installation, open the exact generated output in Hammer, compile it, and inspect it in CS2 before revising the acceptance states.
