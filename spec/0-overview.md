# TADE — Overview

TADE — Typed Annotated Data Exchange — is a byte-stream framing
protocol for typed, annotated data. It defines how typed frames are encoded,
delimited, and carried over an ordered byte stream. It does not define
what those types mean — semantics live in pluggable dialects declared
on each stream.

The name captures what the wire delivers: every frame carries a type
that identifies a data kind — the frame annotates its data with that type — and the exchange is atomic — each
frame is a self-contained, independently decodable unit.

TADE is the substrate layer. Dialects ride on top.

## The core idea

A TADE stream is a sequence of frames. Each frame is four fields:

```
marker  type  size  data
```

- marker — `0x1F` (ASCII Unit Separator); never appears in valid UTF-8
- type — one byte: dialect-level particle kind
- size — LEB128-encoded data length
- data — N bytes; opaque to framing; may contain nested TADE frames

TADE specifies the framing. The type byte is opaque to TADE —
it acquires meaning from whichever dialect is declared on the stream.

## Layering

```
┌──────────────────────────────────────────────────────────┐
│ Application (cyb, agents, renderers like prysm)          │
├──────────────────────────────────────────────────────────┤
│ Dialect (pluggable)                                      │
│   prysm dialect — UI particles for cyberia               │
│   agent dialect — thoughts, tool calls, references       │
│   any other dialect — declared in-stream                 │
├──────────────────────────────────────────────────────────┤
│ TADE Layer 1 — stream control                            │
│   dialect declaration, cancel, heartbeat                 │
├──────────────────────────────────────────────────────────┤
│ TADE Layer 0 — wire framing                              │
│   marker 0x1F, type byte, size varint, data bytes        │
├──────────────────────────────────────────────────────────┤
│ Transport — TCP, WebSocket, stdio, HTTP body, QUIC, file │
└──────────────────────────────────────────────────────────┘
```

TADE owns Layers 0 and 1. Dialects plug into Layer 2.

## Design principles

Bytes only. TADE is a wire protocol. It carries typed bytes; it does
not assign meaning to types. A consumer that knows the wire format can
parse any TADE stream, even if it doesn't know the dialect. The dialect
is what makes those bytes mean something.

Self-describing frames. Each frame carries its own length and type
byte. A decoder can walk a stream without state and skip frames it
doesn't understand.

Resynchronisable. The marker byte `0x1F` cannot appear inside a valid
TADE frame header or in valid UTF-8 data except as part of a nested
frame's marker. A consumer that joins a stream mid-way scans for `0x1F`
and picks up the next complete frame.

Pluggable dialects. A stream declares its dialect at the start. All
subsequent type bytes are interpreted in that dialect's namespace.
Dialects are identified by URN; consumers ship with the dialects they
understand.

Forward-compatible. Decoders MUST skip frames with unrecognised type
bytes rather than failing. Unknown dialects degrade gracefully: stream
control still works (progress, cancel, errors at the stream layer),
application data is skipped.

Transport-agnostic. TADE frames ride on any ordered byte stream.
The protocol defines the frame format; transport is out of scope.

## What TADE does not own

- The meaning of any specific type byte other than the small
  reserved stream-control set (see `2-stream-control.md`).
- Visual rendering of any frame.
- Identity, authentication, content addressing, or capability tokens
  (these belong to particle protocols above TADE).
- Encryption or transport security (use TLS, SSH, WireGuard underneath).
- Session management, flow control, or ordering guarantees beyond what
  the transport provides.

## Scope of this specification

TADE defines:

- Frame encoding ([1-wire-format.md](1-wire-format.md))
- Stream-control namespace ([2-stream-control.md](2-stream-control.md))
- Dialect protocol — how dialects declare themselves and plug in
  ([3-catalog-protocol.md](3-catalog-protocol.md))
- Conformance requirements ([4-conformance.md](4-conformance.md))
- Transport bindings ([5-transport-bindings.md](5-transport-bindings.md))
- Security considerations ([6-security.md](6-security.md))

The cyberia particle dialect (type meanings, data schemas, structural
conventions) is not part of this specification. It is staged in
[7-catalog.md](7-catalog.md) pending migration to its proper home in
`prysm/spec/`.
