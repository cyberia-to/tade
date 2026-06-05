# TAPE — Wire Format

## Frame structure

Every TAPE frame has the following layout:

```
 0        1        2…           N…
 ┌────────┬────────┬─────────────┬────────────────┐
 │ marker │  type  │  size       │  data          │
 └────────┴────────┴─────────────┴────────────────┘
 0x1F      1 byte   1–10 bytes    N bytes
```

### marker

The marker byte is `0x1F` (ASCII Unit Separator). It was chosen because:

- It never appears in valid UTF-8 encoded text (multibyte lead bytes are 0xC0–0xFF)
- It is not in the Nox ISA instruction range (0x01–0x1E)
- It is not a printable ASCII character (0x20–0x7E)
- It is not used by any standard terminal control sequence

A decoder scanning for the start of a frame MUST look for `0x1F`. Bytes before
the first `0x1F` in a stream MUST be discarded without error (allows plain-text
preambles and mixed streams).

### type

One byte identifying the particle kind of this frame. The meaning of each type
value is defined by the active dialect (see [3-catalog-protocol.md](3-catalog-protocol.md)).
The value `0x2A` (`*`) is reserved by TAPE for stream control (see
[2-stream-control.md](2-stream-control.md)). A decoder that receives an unknown
type MUST skip the frame (advance past the data) rather than failing.

### size

Unsigned data length encoded as LEB128 (Little-Endian Base 128):

- Each byte contributes 7 bits of the value, LSB first.
- The high bit of each byte is 1 if more bytes follow, 0 for the last byte.
- Maximum encoded value: 2^63 − 1 (10 bytes maximum).
- A decoder that reaches 10 bytes without a terminating byte MUST treat the
  frame as malformed and skip forward to the next `0x1F`.

Examples:

| Value | Bytes |
|-------|-------|
| 0 | `00` |
| 1 | `01` |
| 127 | `7F` |
| 128 | `80 01` |
| 300 | `AC 02` |
| 16383 | `FF 7F` |
| 16384 | `80 80 01` |

### data

Exactly N bytes as given by size. The data is opaque to the framing
layer — its interpretation depends on the type byte and the active dialect.
For composite particle types (component, table, struct), the data is itself
a sequence of valid TAPE frames that can be decoded with the same parser.

## Encoding algorithm

```
fn encode(type: u8, data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1F, type];
    encode_leb128(data.len() as u64, &mut out);
    out.extend_from_slice(data);
    out
}
```

## Decoding algorithm

```
fn decode_next(buf: &[u8], pos: &mut usize) -> Option<Frame> {
    // scan for marker
    let start = buf[*pos..].iter().position(|&b| b == 0x1F)?;
    *pos += start;
    if buf.len() - *pos < 3 { return None; }  // need marker + type + size(min 1)
    let frame_type = buf[*pos + 1];
    *pos += 2;
    let (data_len, new_pos) = decode_leb128(buf, *pos)?;
    *pos = new_pos;
    if buf.len() - *pos < data_len { return None; }  // incomplete
    let data = &buf[*pos .. *pos + data_len];
    *pos += data_len;
    Some(Frame { frame_type, data })
}
```

A decoder MUST handle:
- Multiple frames concatenated end-to-end in a single buffer
- Frames split across multiple `feed()` calls (incremental / streaming)
- Unknown type: skip (advance past data) without error
- Truncated frame at end of buffer: return Pending, not an error

## Minimum overhead

| Data size | Total frame bytes |
|---|---|
| 0 | 3 (marker + type + `0x00`) |
| 1–127 | 4 |
| 128–16383 | 5 |
| 16384–2097151 | 6 |

A 100-byte particle costs 4 bytes of overhead (4%).
