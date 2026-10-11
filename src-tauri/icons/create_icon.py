import struct, zlib

# Create a valid 1x1 RGB PNG

# 1. PNG file signature
sig = b'\x89PNG\r\n\x1a\n'

# 2. IHDR chunk: width=1, height=1, bit_depth=8, color_type=2 (RGB)
# IHDR data packed as: >IIBBBBB = 2 ints + 5 bytes = 13 bytes
ihdr_data = struct.pack('>IIBBBBB', 1, 1, 8, 2, 0, 0, 0)
# CRC of IHDR: CRC32 of ("IHDR" + data)
ihdr_crc = struct.pack('>I', zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff)
# IHDR chunk: length(4) + type(4) + data(13) + CRC(4) = 25 bytes
ihdr_chunk = struct.pack('>I', 13) + b'IHDR' + ihdr_data + ihdr_crc

# 3. IDAT chunk: filter byte (0) + pixel (R=255, G=0, B=0) compressed
raw_data = b'\x00\xff\x00\x00'
compressed_data = zlib.compress(raw_data)
# IDAT chunk: length(4) + type(4) + data(N) + CRC(4)
idat_chunk = b'IDAT' + compressed_data + struct.pack('>I', zlib.crc32(b'IDAT' + compressed_data) & 0xffffffff)

# 3. IEND chunk: length=0, type="IEND", CRC of "IEND"
iend_chunk = b'IEND' + struct.pack('>I', zlib.crc32(b'IEND') & 0xffffffff)

# 4. Combine: signature + IHDR + IDAT + IEND
png_data = sig + ihdr_chunk + idat_chunk + iend_chunk

# Write to file
with open('icon.png', 'wb') as f:
    f.write(png_data)

print("Created icon.png")
