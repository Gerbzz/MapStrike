#!/usr/bin/env bash
set -euo pipefail

root="${RUNNER_TEMP}/mapstrike-native-visual"
mkdir -p "$root"
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
export XDG_CONFIG_HOME="${root}/config"
cli="./target/debug/mapstrike"
native="./target/debug/mapstrike-capture"
document="${root}/synthetic-room.json"
capture="${root}/blockout.png"

"$cli" new-blockout "Synthetic room study" "$document"
"$cli" add-room "$document" -512 -512 0 1024 1024 256 16
"$cli" add-block "$document" 0 0 32 192 192 96
"$cli" add-spawn "$document" t -256 0 16 0
"$cli" add-spawn "$document" ct 256 0 16 180
timeout 50s xvfb-run -a -s '-screen 0 1600x1000x24' "$native" "$document" "$capture"

python3 - "$capture" <<'PY'
import hashlib
import pathlib
import struct
import sys
import zlib

path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
if len(data) < 5000 or not data.startswith(b"\x89PNG\r\n\x1a\n"):
    raise SystemExit("native capture is missing or incomplete")
width, height = struct.unpack(">II", data[16:24])
if width < 1000 or height < 700:
    raise SystemExit(f"unexpected native capture dimensions: {width}x{height}")
offset = 8
compressed = bytearray()
while offset < len(data):
    length = struct.unpack(">I", data[offset:offset + 4])[0]
    kind = data[offset + 4:offset + 8]
    if kind == b"IDAT": compressed.extend(data[offset + 8:offset + 8 + length])
    offset += length + 12
if len(set(zlib.decompress(compressed))) < 24:
    raise SystemExit("native capture lacks visible content")
print(f"blockout: {width}x{height}, {len(data)} bytes, SHA-256 {hashlib.sha256(data).hexdigest().upper()}")
PY
