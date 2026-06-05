# TAPE: Typed Annotated Payload Exchange

**A framing and typing protocol for structured byte streams**

2026

---

## Abstract

Terminal output has not changed since 1978. ANSI escape codes give us color
and cursor control but throw away all semantic meaning: a table looks like a
table on screen but arrives as a string of characters with no machine-readable
structure. JSON-RPC and Protocol Buffers solve the schema problem but require
out-of-band schema distribution and impose significant encoding overhead.
MessagePack is compact but still schema-free. gRPC is fast but HTTP/2-only
and binary-schema-required.

TAPE fills a specific gap: **a framing protocol that carries typed, structured
output over any ordered byte stream with zero schema distribution**. Every
frame declares its own semantic role (sigil) and perception type (render) in
two ASCII bytes. A table is a table on the wire, not a string that happens to
look like one. An error is an error, not a line that might start with "ERROR:".

TAPE is designed for the terminal era we are actually in: AI agents producing
structured output, CLI tools piping typed data, distributed systems logging to
stream consumers that can render, filter, and forward without parsing.

---

## 1. Motivation

### 1.1 The terminal's forgotten semantics

The VT100 terminal (1978) defined a vocabulary for controlling a character
display. ANSI escape codes extended that vocabulary with color, bold, underline.
But the vocabulary was always about *appearance*, never about *meaning*.

A modern CLI tool that outputs a table must choose: either print human-readable
text that a human can read and a machine cannot parse, or print JSON that a
machine can parse and a human cannot read. There is no third option in the
current ecosystem.

TAPE is that third option.

### 1.2 Why existing protocols do not solve this

**ANSI escape codes**: appearance only. A progress bar is rendered by moving
the cursor up and overwriting a line. A machine consumer sees escape sequences
it must reverse-engineer. No semantic type is ever transmitted.

**JSON-RPC**: requires a client/server model. Every message must be a JSON
object with `method`, `params`, and `id` fields. The protocol envelope costs
more bytes than most payloads. Streaming is bolted on awkwardly. JSON is
human-readable at the cost of parsing overhead and UTF-8 constraints.

**Protocol Buffers / gRPC**: fast and compact, but require a `.proto` schema
compiled into both producer and consumer. Distributing schemas is operational
overhead that most tools and agent systems cannot assume. gRPC adds HTTP/2
framing on top.

**MessagePack**: compact binary JSON. Still requires a schema or runtime type
introspection. No semantic type tags.

**MCP (Model Context Protocol)**: JSON-RPC-based, carries tool call results as
text or embedded JSON. A table arrives as a JSON array of objects. The consumer
must know the schema to know it is a table.

**NDJSON**: one JSON object per line. Human-readable, but every frame pays full
JSON overhead, and line buffering breaks streaming on partial lines.

### 1.3 The AI agent problem

AI agents are the sharpest edge of this problem. An agent produces a response
that contains:

- Progress updates ("thinking… searching… generating…")
- Structured data (tables, trees, key-value results)
- Text (prose, code blocks)
- Errors (with source and severity)
- Actions (buttons the user can click)
- Status (done, exit code)

Current systems serialize all of this to Markdown strings and hope the
consumer knows how to parse it back. TAPE lets the agent emit each chunk
as its actual type. The terminal renders them correctly. A downstream agent
receives them as typed data without parsing.

---

## 2. Design principles

### 2.1 The wire is the schema

TAPE frames carry their type in the frame header. A consumer that has never
seen a producer can decode any frame. There is no schema registry, no
`.proto` file, no OpenAPI document. The 256-byte map (§2.2) is the schema,
and it fits in one page.

### 2.2 The 256-byte map

Every possible byte value is assigned a role exactly once:

| Range | Role |
|-------|------|
| 0x01–0x1E | Nox ISA instruction space (reserved) |
| 0x1F | TAPE frame marker |
| 0x20–0x7E | Printable ASCII: sigils and render types |
| 0x7F | DEL (reserved) |
| 0x80–0xFF | UTF-8 continuation/lead bytes |

This means 0x1F can only appear as a frame start byte in a valid TAPE stream.
No payload byte, no varint byte, no sigil or render byte can be 0x1F. Frame
boundaries are unambiguous without escaping.

### 2.3 Sigil and render: two axes of meaning

Every frame has a **sigil** (semantic role) and a **render** (perception type).

The sigil answers: *what is this for?*
- `#` HAX — content / primary data
- `!` ZAP — effect / action / error
- `.` DOT — pipeline / transform / meta
- `~` SIG — annotation / side-info
- `=` TIS — binding / key-value
- `@` PAT — identity / agent / reference
- `|` BAR — composition / juxtaposition
- `/` FAS — scope / hierarchy
- `:` COL — pair / association
- `?` WUT — test / input
- `$` BUC — economic / token value
- `+` LUS — augmentation / delta
- `^` KET — abstraction / definition

The render answers: *how should this be perceived?*
- `t` TEXT — prose or code
- `T` TABLE — 2D grid
- `e` ERROR — typed error with source
- `l` LOG — structured log line
- `p` PROGRESS — live-updatable progress bar
- `x` STATUS — exit code sentinel
- `c` COMPONENT — nested composition
- `s` STRUCT — collapsible tree
- `b` BINARY — raw bytes / image
- `v` VECTOR — SVG / geometry
- `a` AUDIO — waveform
- `m` MOVIE — video frames
- `f` FORMULA — mathematical notation
- `i` INPUT — user response
- `k` TOKEN — ledger entry

A plain text chunk is `(#, t)`. An error is `(!, e)`. A progress bar is
`(., p)`. These are not arbitrary — the sigil and render each carry
independent meaning that composes.

### 2.4 Zero overhead composition

Frames compose by nesting: the payload of any frame is itself a valid TAPE
byte stream. A table is a `(#, T)` frame whose payload contains schema and
row frames. A key-value pair is a `(=, s)` frame whose payload contains a
key annotation and a value frame of any type.

There is no envelope, no container header, no length-of-lengths. Nesting is
free: the outer frame's varint length already covers the inner frames.

### 2.5 Transport agnosticism

TAPE is a byte-stream protocol. It makes no assumptions about the transport.
The same frame format works over:

- Unix stdin/stdout pipes
- TCP sockets
- WebSocket binary messages
- HTTP streaming response bodies
- Unix domain sockets
- Files

The frame marker 0x1F enables resynchronization: a consumer that joins a
stream mid-way scans for 0x1F and picks up the next complete frame. Junk
bytes before the first frame marker are silently discarded.

### 2.6 Forward compatibility by default

A consumer that encounters an unknown sigil or render byte skips the frame.
This is a MUST in the conformance spec, not a SHOULD. It means private
extensions can be deployed without breaking existing consumers.

---

## 3. Wire format

A TAPE frame is:

```
┌──────────┬───────────┬──────────────┬──────────────────┐
│  MARKER  │   SIGIL   │    RENDER    │    PAYLOAD       │
│  0x1F    │ 0x20–0x7E │  0x20–0x7E   │ varint(N) + N B  │
│  1 byte  │  1 byte   │   1 byte     │  1–10 + N bytes  │
└──────────┴───────────┴──────────────┴──────────────────┘
```

The payload length is encoded as unsigned LEB128 (little-endian base-128
variable-length integer). Most payloads are under 128 bytes; those pay 1
extra byte for the length. A 16 KB payload pays 3 bytes. The minimum frame
is 4 bytes (marker + sigil + render + zero-length varint).

### 3.1 Encoding overhead

| Payload size | Frame overhead | Ratio |
|-------------|---------------|-------|
| 0 B | 4 B | — |
| 10 B | 4 B | 40% |
| 100 B | 4 B | 4% |
| 1 KB | 4 B | <1% |
| 16 KB | 5 B | <1% |
| 2 MB | 6 B | <1% |

For typical AI output (prose + structured data), TAPE adds 3–5% overhead
compared to bare text. This is less than JSON's structural overhead.

---

## 4. Comparison

| Protocol | Schema required | Streaming | Semantic types | Wire overhead | Transport |
|----------|----------------|-----------|---------------|--------------|-----------|
| ANSI escape | no | yes | no (visual only) | ~5% | terminal |
| JSON-RPC | implicit | awkward | no | 50–200% | any |
| Protocol Buffers | yes (.proto) | yes | yes | 5–20% | any |
| MessagePack | implicit | yes | no | 10–30% | any |
| MCP | implicit (JSON-RPC) | yes | no | 50–200% | SSE/HTTP |
| NDJSON | implicit | yes | no | 30–100% | any |
| **TAPE** | **no** | **yes** | **yes** | **<5%** | **any** |

Key differentiator: TAPE is the only protocol in this table that is
simultaneously schema-free, semantically typed, and streaming-native on
any transport.

---

## 5. Defined chunk types

The complete type catalog:

| Pair | Name | Use |
|------|------|-----|
| `(#, t)` | Text | Plain UTF-8 prose or code |
| `(#, T)` | Table | 2D grid with header row |
| `(@, t)` | Neuron | Identity reference (`@name`) |
| `(~, t)` | Annotation | Label or side-info metadata |
| `(!, e)` | Error | Structured error with level, source, message |
| `(!, c)` | Action | Button: label + target reference |
| `(., l)` | Log | Structured log: level, source, message |
| `(., p)` | Progress | Live progress: id, label, current, total |
| `(., x)` | Status | End-of-command: exit code |
| `(\|, c)` | Component | Vertical stack of child frames |
| `(/, c)` | Scope | Path/breadcrumb container |
| `(/, s)` | Schema | Table header row |
| `(:, s)` | Row | Table data row |
| `(=, s)` | KV pair | Key `(~, t)` + value frame |
| `(?, c)` | Input request | Prompt for user input |
| `(?, i)` | Input response | Raw user input text |
| `($, t)` | Economic text | Currency/token value as text |
| `($, k)` | Token | Ledger entry / balance |
| `(+, t)` | Augmentation | Delta or additive text |

19 chunk types cover the full range of terminal and agent output.

---

## 6. The kv convention

Meta chunks (`(., l)`, `(!, e)`, `(., p)`, `(., x)`) all use the same
payload layout: a sequence of `(=, s)` key-value frames, each containing
a `(~, t)` key and a value frame.

This means structured fields (error level, log source, progress label) are
encoded in the same type system as the payloads they describe. There is no
separate "metadata encoding" — TAPE is self-describing at every level.

Example: a structured error on the wire

```
0x1F 0x21 0x65  [payload bytes]
  └── (!, e)
      payload:
        0x1F 0x3D 0x73  [kv: level = "error"]
          0x1F 0x7E 0x74 0x05 "level"   ← (~, t) key
          0x1F 0x23 0x74 0x05 "error"   ← (#, t) value
        0x1F 0x3D 0x73  [kv: message = "file not found"]
          0x1F 0x7E 0x74 0x07 "message"
          0x1F 0x23 0x74 0x0E "file not found"
```

---

## 7. Security model

TAPE provides no cryptographic services. It is a framing protocol, not a
security protocol. Confidentiality, integrity, authentication, and replay
prevention are the responsibility of the transport layer (TLS, SSH,
WireGuard).

The primary security considerations for TAPE implementations are
denial-of-service defenses:

- **Frame size limit**: consumers SHOULD enforce a maximum frame size
  (recommended: 64 MiB).
- **Nesting depth limit**: consumers SHOULD enforce a maximum nesting depth
  (recommended: 64 levels).
- **varint overflow**: a varint longer than 10 bytes or encoding a value
  larger than 2⁶³ − 1 MUST be rejected.

TAPE payloads are opaque bytes. Consumers that render payloads as HTML, SQL
queries, or shell commands MUST sanitize appropriately for their rendering
context.

---

## 8. Reference implementation

The reference implementation is a Rust crate at `impl/rust/`. It provides:

- `Chunk` — raw frame (sigil, render, payload bytes)
- `Writer` — frame encoder to any `Write`
- `Reader` — streaming frame decoder from any `Read`
- `Molecule` — typed enum: `from_chunk(&Chunk)` / `to_chunk(&self)`
- Helpers: `encode_nested`, `decode_nested`, `kv`, `read_kv`, `table_chunk`

The crate has one dependency: `bytes` for zero-copy buffer management.

```rust
use tape::{Writer, Molecule, Text, Status};
use std::io::stdout;

let mut w = Writer::new(stdout());
w.write_molecule(&Molecule::Text(Text { content: "hello, tape".into() }))?;
w.write_molecule(&Molecule::Status(Status { code: 0 }))?;
```

---

## 9. Conformance

A conformant implementation MUST pass all vectors in
`conformance/vectors/`. The conformance suite covers 14 required vectors:
plain text, annotation, zero/nonzero status, error, log, progress, table,
component, kv pair, multi-frame, junk-before-marker, truncated payload, and
unknown sigil.

See `spec/4-conformance.md` for the full test vector specification.

---

## 10. Relationship to the cyber ecosystem

TAPE is the byte-stream substrate for cyberia tooling. The sigil vocabulary
(cybermark) is shared across the cyber symbol system — the same 13 sigils
that organize the TAPE type catalog appear in cyberlinks, particle types, and
the Nox ISA.

TAPE is designed to be cyberia-independent. The protocol specification
(this document + `spec/`) makes no reference to cyberia, prysm, or any
upstream system. Any application can adopt TAPE without adopting the broader
cyber stack.

---

## Appendix A: Sigil etymology

The sigil names are drawn from Hoon/Nock tradition, where each ASCII character
has a two-syllable name. The names are mnemonic, not meaningful:

| Char | Name | Mnemonic |
|------|------|----------|
| `!` | ZAP | bang / exclamation / effect |
| `#` | HAX | hash / content |
| `$` | BUC | buck / economic |
| `+` | LUS | plus / augment |
| `.` | DOT | dot / pipeline |
| `/` | FAS | slash / scope |
| `:` | COL | colon / pair |
| `=` | TIS | tis / equals / binding |
| `?` | WUT | what / question / test |
| `@` | PAT | at / identity |
| `^` | KET | ket / lift / abstract |
| `\|` | BAR | bar / pipe / compose |
| `~` | SIG | sig / annotation |

---

## Appendix B: LEB128 encoding reference

Unsigned LEB128 encodes a non-negative integer as a sequence of 7-bit groups,
least significant first, with the high bit of each byte set to 1 except the
last byte.

```
value 0:       0x00
value 127:     0x7F
value 128:     0x80 0x01
value 255:     0xFF 0x01
value 16383:   0xFF 0x7F
value 16384:   0x80 0x80 0x01
value 2^63-1:  0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0x7F
```

A varint longer than 10 bytes or encoding a value larger than 2⁶³ − 1 is
malformed and MUST be rejected.

---

*See `spec/` for the normative wire format documents.*
