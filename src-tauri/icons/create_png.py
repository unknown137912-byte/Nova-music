import struct
import zlib
from pathlib import Path

def make_chunk(chunk_type, data):
    crc = zlib.crc32(chunk_type + data) & 0xffffffff
    return struct.pack(">I", len(data)) + chunk_type + data + struct.pack(">I", crc)

signature = b"\x89PNG\r\n\x1a\n"

ihdr = struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0)
raw_data = b"\x00\x00\x00\x00\x00"

png_data = (
    signature
    + make_chunk(b"IHDR", ihdr)
    + make_chunk(b"IDAT", zlib.compress(raw_data))
    + make_chunk(b"IEND", b"")
)

Path("icon.png").write_bytes(png_data)

# Verify the PNG chunk structure and CRCs.
data = Path("icon.png").read_bytes()
assert data.startswith(signature), "Invalid PNG signature"

pos = len(signature)
chunks = []

while pos < len(data):
    length = struct.unpack(">I", data[pos:pos + 4])[0]
    chunk_type = data[pos + 4:pos + 8]
    chunk_data = data[pos + 8:pos + 8 + length]
    crc = struct.unpack(">I", data[pos + 8 + length:pos + 12 + length])[0]

    assert len(chunk_data) == length, "Truncated PNG chunk"
    assert zlib.crc32(chunk_type + chunk_data) & 0xffffffff == crc, (
        f"CRC mismatch: {chunk_type!r}"
    )

    chunks.append(chunk_type.decode("ascii"))
    pos += 12 + length

assert pos == len(data), "Unexpected trailing data"
assert chunks == ["IHDR", "IDAT", "IEND"], f"Unexpected chunks: {chunks}"

print("SUCCESS: Valid PNG!")
print("Chunks:", chunks)
print("Generated:", Path("icon.png").resolve())
print("Note: This is a 1x1 transparent test image, not the final app icon.")
