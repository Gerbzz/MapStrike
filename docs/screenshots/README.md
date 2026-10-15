# MapStrike native renderer capture provenance

`mapstrike-blockout-prototype.png` is an unedited 650 x 370 PNG from the running native egui `MapStrikeApp`. It shows the current form-based blockout prototype with four synthetic boxes. It is not a browser recreation or a 3D map preview.

- App UI source: public `main` commit [`aef7c1932479297688eb247cbe700b30c8bd6ede`](https://github.com/Gerbzz/MapStrike/commit/aef7c1932479297688eb247cbe700b30c8bd6ede). The UI itself was not modified for this capture.
- Evidence-only helper: commit [`c8edb27c00727660cd1adf84ae3b677a2bce7471`](https://github.com/Gerbzz/MapStrike/commit/c8edb27c00727660cd1adf84ae3b677a2bce7471) on [draft PR #1](https://github.com/Gerbzz/MapStrike/pull/1). This capture binary loads a saved blockout through `MapStrikeApp::open_blockout`, asks egui for a screenshot, writes a new PNG, and closes. The helper is not part of the current `main` binary.
- Host and date: Office PC, 2026-10-15 UTC. Cargo 1.97.1 and rustc 1.97.1. Build was offline, single job, with debug symbols and incremental compilation disabled. Existing cached eframe dependencies were reused. C: had 21.09 GiB free before this build and 20.93 GiB after capture, above the 20 GiB build floor.
- Capture binary SHA-256: `ba0caad4a4d12ae2b3b676538883a3151d243e462bca029d7a741403bfcb875c`.
- CLI binary SHA-256: `6768667c9e8c527fd5e92767601699e3aec524799fe86f8d4c154315f74c28ad`.
- [Synthetic fixture](fixtures/courtyard-study.json) SHA-256: `2f93d3caf97fa97e3f27ed7e97951e611d58f2f0911e9900a1b3b9f31cc5fd54`. The first-party CLI created the named blockout and added four axis-aligned boxes; it contains no outside assets or user data.
- [Final PNG](mapstrike-blockout-prototype.png) SHA-256: `7b19eefc527aff0281d75f32e2836f6c3d37269466b1d813aa4e3a70f6f561d7`.
- Verification: the native-feature suite passed 7/7 tests, including the capture helper's argument test, using a workspace temporary directory. The final PNG was opened and visually inspected after the capture.

The capture helper was built with:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Neil\Documents\Codex\2026-10-14\task-3\ModelStrike\target'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
cargo build --locked --offline --features native-ui --bin mapstrike-capture --bin mapstrike -j 1
```

The synthetic data and final PNG were produced with the first-party binaries. The final helper is configured for a 650 x 370 capture window:

```powershell
$exe='C:\Users\Neil\Documents\Codex\2026-10-14\task-3\ModelStrike\target\debug\mapstrike.exe'
$doc='docs\screenshots\fixtures\courtyard-study.json'
& $exe new-blockout 'Courtyard Study' $doc
& $exe add-block $doc 0 0 0 512 512 32
& $exe add-block $doc 0 0 32 64 512 160
& $exe add-block $doc 320 224 32 512 416 96
& $exe add-block $doc 160 160 32 352 224 128
& 'C:\Users\Neil\Documents\Codex\2026-10-14\task-3\ModelStrike\target\debug\mapstrike-capture.exe' $doc target\mapstrike-preview-final.png
Copy-Item -LiteralPath target\mapstrike-preview-final.png -Destination docs\screenshots\mapstrike-blockout-prototype.png
```

This frame does not establish pointer or keyboard interaction, file-picker operation, saving or exporting through the UI, map geometry visualization, game compatibility, or clean-machine packaging.
