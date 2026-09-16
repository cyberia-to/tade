# TADE — Dialect Protocol

A dialect is a named vocabulary of type bytes along with their semantic
identities, data schemas, and structural conventions. TADE itself defines
no dialects — they are external specifications.

This document specifies how dialects declare themselves on a TADE
stream and how multiple dialects coexist.

## What a dialect is

A dialect defines:

- A namespace of type byte codes (subject to the reservation in
  [2-stream-control.md](2-stream-control.md): `0x2A` (`*`) is reserved by TADE).
- The semantic identity of each defined code (e.g. "this is a
  structured error", "this is a neuron identity reference").
- The data encoding for each code (e.g. UTF-8 text, kv struct
  with these required keys, nested frames with this layout).
- Any structural conventions the dialect imposes (e.g. ordering of
  child frames inside a composite data field).

A dialect does not define:

- Visual rendering. Display is the renderer's choice. The same
  dialect can be rendered by prysm one way, by a CLI fallback
  another way, by an audit-log archiver a third way.
- Transport. Dialects are transport-agnostic; TADE handles transport
  bindings.
- Wire framing. The frame format is fixed by TADE.

## Dialect identifiers

A dialect identifier is a UTF-8 string in URN form:

```
urn:<authority>:<name>:<version>
```

| Component | Description |
|-----------|-------------|
| `authority` | A namespace owner (e.g. `cyberia`, `acme`, an organisation, an agent system) |
| `name` | The dialect name (e.g. `prysm`, `agent-comm`, `research`) |
| `version` | Major version number; minor/patch versions are implementation detail |

Examples:

```
urn:cyberia:prysm:1
urn:cyberia:agent:1
urn:acme:tickets:2
```

Implementations MUST treat dialect identifiers as opaque strings for
matching purposes. Two identifiers are the same dialect if and only if
their byte sequences are identical.

## Declaration

A TADE stream declares its dialect by emitting a `(*, k)` frame
(see [2-stream-control.md](2-stream-control.md)) with the dialect
identifier as data. The declaration SHOULD be the first frame on
the stream.

```
0x1F 0x2A 0x6B  <size-varint>  "urn:cyberia:prysm:1"
```

After declaration, every subsequent non-`*` frame is interpreted in
the declared dialect's namespace until either:

- A new `(*, k)` is emitted (dialect switch), or
- The stream ends.

## Streams without a declaration

A stream MAY omit the `(*, k)` declaration. In that case the consumer:

- MUST process `(*, *)` stream-control frames as defined by TADE.
- MUST skip all other frames (dialect-unknown).
- MAY apply a default dialect out-of-band (e.g. configured at the
  consumer); this is implementation-specific and not portable.

Producers SHOULD declare a dialect explicitly. Streams without
declarations are intended for diagnostic / raw inspection use cases.

## Dialect switching

A `(*, k)` frame emitted mid-stream switches the active dialect for
all subsequent non-`*` frames. There is no "stack" of dialects — at
most one dialect is active at a time, and a new declaration fully
replaces the previous one.

Producers SHOULD avoid frequent switching; if a stream needs
particles from two dialects, prefer one dialect that re-exports
the relevant types, or use separate streams.

## Unknown dialects

A consumer that receives a `(*, k)` declaration for a dialect it does
not know MUST:

- Continue processing stream-control frames (`(*, *)`).
- Skip all other frames (the entire dialect is unknown).
- Remain available for a possible later switch to a known dialect.

The consumer SHOULD log or surface the unknown-dialect condition for
diagnostic purposes; it MUST NOT treat unknown-dialect as a fatal
error.

## Multiple dialects in one ecosystem

The cyberia ecosystem ships with a reference dialect (currently bundled
with prysm, identifier `urn:cyberia:prysm:1`). Other dialects may
coexist:

- `urn:cyberia:agent:1` — agent-to-agent communication primitives
  (thoughts, tool calls, references, citations).
- `urn:cyberia:research:1` — research-pipeline primitives (claims,
  evidence, sources).
- Application-private dialects under any authority.

A consumer can support any subset; unsupported dialects degrade to
dialect-unknown handling.

## Versioning

Dialects version independently of TADE. A v1 dialect and a v2 dialect
of the same name are distinct dialects from TADE's perspective: their
identifiers are different strings, so frames are not interchangeable.

Dialect authors are responsible for documenting compatibility between
versions of their own dialects.

## Dialect conformance

A dialect specification SHOULD include its own conformance section
covering producer / consumer requirements for the codes it defines.
TADE's conformance ([4-conformance.md](4-conformance.md)) covers only
wire-level behaviour; dialect-level conformance is the dialect's
responsibility.
