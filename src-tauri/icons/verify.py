import struct, zlib

with open('icon.png', 'rb') as f:
    data = f.read()

print(f'File size: {len(data)} bytes')
print(f'Signature: {data[:8]}')

# Parse chunks starting after the 8-byte signature
pos = 8
parsed_chunks = []
while pos < len(data) - 4:
    chunk_len = struct.unpack('>I', data[pos:pos+4])[0]
    chunk_type = data[pos+4:pos+8]
    parsed_chunks.append(chunk_type)
    pos += 12 + chunk_len

print(f'Chunks: {[c.decode("ascii", errors="replace") for c in parsed_chunks]}')
if parsed_chunks == [b'IHDR', b'IDAT', b'IEND']:
    print('VALID PNG!')
else:
    print('INVALID PNG')
