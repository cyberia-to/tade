# TADE — Conformance

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in this document
are to be interpreted as described in
[RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

TADE's conformance covers wire-level behaviour only: framing, size varint
encoding, marker handling, stream-control frames, and dialect declaration.
Dialect-level conformance (data schemas, structural conventions for
specific type codes) is the responsibility of each dialect's
own specification.

## Producer requirements

A conformant producer MUST:

- Emit `0x1F` as the first byte of every frame.
- Emit exactly one type byte (0x00–0xFF) as the second byte.
- Encode data length as unsigned LEB128 immediately following the
  type byte.
- Emit exactly `N` data bytes where `N` is the encoded length.
- Use the reserved type byte `*` (0x2A) only for the stream-control
  frames defined in [2-stream-control.md](2-stream-control.md).

A conformant producer SHOULD:

- Emit a `(*, k)` dialect declaration as the first frame of the stream.
- Not emit frames larger than 64 MiB unless the application explicitly
  requires it (see [6-security.md](6-security.md)).

## Consumer requirements

A conformant consumer MUST:

- Scan for `0x1F` to find frame boundaries; discard bytes before the
  first marker.
- Handle frames split across multiple read calls (incremental /
  streaming decoding).
- Skip frames with unrecognised type bytes without signalling an error.
- Skip frames with fewer data bytes than declared (truncated at end
  of stream): treat as Pending.
- Recognise the reserved type byte `*` as TADE stream control and process
  `(*, *)` frames per [2-stream-control.md](2-stream-control.md).
- Treat a stream without a `(*, k)` declaration as dialect-unknown: skip
  all non-`*` frames.
- Treat a `(*, k)` declaration for an unknown dialect as dialect-unknown:
  continue stream-control handling, skip all non-`*` frames until the
  next declaration or end of stream.

A conformant consumer SHOULD:

- Surface unknown-dialect conditions diagnostically without aborting.
- Switch active dialect when a new `(*, k)` is observed.
- Fall back to raw display of unknown type codes rather than hiding
  them entirely (useful for diagnostic tools).

## Varint encoding

A conformant implementation MUST encode and decode unsigned LEB128 as
specified in [1-wire-format.md](1-wire-format.md):

- Values 0–127 use exactly one byte.
- Each continuation byte has the high bit set; the terminal byte has the
  high bit clear.
- A varint longer than 10 bytes, or one encoding a value larger than
  2⁶³ − 1, MUST be rejected as malformed. The decoder MUST advance past
  the malformed varint by scanning for the next `0x1F`.

## Resynchronisation

A conformant consumer MUST be able to resynchronise after malformed
input:

- On any decode error (bad varint, declared length exceeds available
  bytes within the current readable buffer when reading from a finite
  source), the consumer MUST discard bytes until the next `0x1F` and
  attempt to decode from there.
- Resynchronisation MUST NOT cause the consumer to silently report a
  successful decode for malformed input.

## Test vectors

Test vectors live in `../conformance/vectors/`. Each vector is a pair:

- `{name}.bin` — raw TADE bytes
- `{name}.json` — JSON descriptor of expected decoded structure

### Descriptor format

```json
{
  "name": "dialect_declaration",
  "description": "First frame declares dialect",
  "frames": [
    {
      "type": 42,
      "data_utf8": "urn:cyberia:prysm:1"
    }
  ]
}
```

For nested frames, use `"data_frames"` instead of `"data_utf8"`.
Nested frames are still framed bytes; TADE's wire spec does not
interpret them, but conformance vectors may use nesting to exercise
the recursive decode path.

### Required vectors

An implementation MUST pass all vectors in `conformance/vectors/`:

| Vector | Tests |
|--------|-------|
| `dialect_decl.bin` | `(*, k)` with a dialect URN as data |
| `cancel.bin` | `(*, c)` with a reason as data |
| `cancel_empty.bin` | `(*, c)` with empty data |
| `heartbeat.bin` | `(*, h)` with empty data |
| `unknown_type.bin` | Frame with an unknown type byte — must be skipped |
| `multi_frame.bin` | Three frames concatenated, including a dialect declaration |
| `junk_before_marker.bin` | 5 junk bytes then a valid frame (junk discarded) |
| `truncated.bin` | Data declared as 10 bytes but only 5 provided (Pending) |
| `bad_varint.bin` | Varint longer than 10 bytes — must resynchronise to next `0x1F` |
| `nested_data.bin` | A frame whose data contains two complete frames (recursive decode) |

Dialect-specific vectors (errors, tables, kv pairs with specific keys)
belong to the dialect's own conformance suite, not TADE's.

### Running conformance tests

```bash
# Rust reference implementation
cargo test --manifest-path impl/rust/Cargo.toml conformance

# Any implementation
tade-conformance --vectors conformance/vectors/
```
