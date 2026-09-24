# Staged: Catalog content pending migration

> **This file is a staging area, not a TADE specification.**
>
> The content below was previously in `2-type-catalog.md` and
> `3-composition.md`. It is **catalog-level** content: semantic identity of
> specific `(sigil, form)` byte pairs, payload schemas, and structural
> conventions used by the cyberia particle vocabulary.
>
> TADE itself is a byte-stream framing protocol and owns none of this. The
> meaning of `(sigil, form)` codes belongs to whichever catalog is declared
> on a stream. The cyberia catalog (currently bundled with prysm) is one
> such catalog.
>
> This file collects all the moved semantics into one place pending a
> decision on where it should ultimately live — most likely
> `prysm/spec/`, possibly a separate `catalog/` repo.

---

## 1. Sigils — semantic role bytes

The cyberia catalog uses 13 printable-ASCII bytes as semantic role tags,
drawn from cybermark.

| Byte | Char | Name | Semantic role |
|------|------|------|---------------|
| 0x21 | `!`  | ZAP  | Effect / imperative / action — something that causes change |
| 0x23 | `#`  | HAX  | Content / particle identity — primary data |
| 0x24 | `$`  | BUC  | Economic / value-bearing — currency, tokens, balances |
| 0x28 | `+`  | LUS  | Augment / increment — additive delta |
| 0x2E | `.`  | DOT  | Transform / pipeline / apply — process-control |
| 0x2F | `/`  | FAS  | Scope / containment / structure — hierarchical grouping |
| 0x3A | `:`  | COL  | Pair / key-value — association |
| 0x3D | `=`  | TIS  | Binding / equivalence — named value |
| 0x3F | `?`  | WUT  | Test / decision / input request — uncertainty |
| 0x40 | `@`  | PAT  | Identity / agent / neuron — named entity |
| 0x5E | `^`  | KET  | Lift / abstract / establish — definition |
| 0x7C | `\|` | BAR  | Composition / code-with-data — juxtaposition |
| 0x7E | `~`  | SIG  | Annotation / label / side-info — metadata |

A consumer in this catalog skips frames whose sigil is outside the set.

## 2. Form bytes — payload structure / perception

The cyberia catalog uses 15 ASCII alphabetic bytes as form tags.

| Byte | Char | Name      | Form |
|------|------|-----------|------|
| 0x54 | `T`  | TABLE     | 2D grid of records |
| 0x61 | `a`  | AUDIO     | Waveform / audio samples |
| 0x62 | `b`  | BINARY    | Raster image / raw bytes |
| 0x63 | `c`  | COMPONENT | Nested composition (payload contains frames) |
| 0x65 | `e`  | ERROR     | Typed error with source location |
| 0x66 | `f`  | FORMULA   | Mathematical notation |
| 0x69 | `i`  | INPUT     | User → producer response channel |
| 0x6B | `k`  | TOKEN     | Ledger entry / balance |
| 0x6C | `l`  | LOG       | Structured log line |
| 0x6D | `m`  | MOVIE     | Video frames / animation |
| 0x70 | `p`  | PROGRESS  | Live progress bar (live-updatable) |
| 0x73 | `s`  | STRUCT    | Collapsible tree / structured data |
| 0x74 | `t`  | TEXT      | Prose / code / plain text |
| 0x76 | `v`  | VECTOR    | SVG / 2D / 3D geometry |
| 0x78 | `x`  | STATUS    | End-of-command sentinel (exit code) |

## 3. Defined chunk types

| Pair | Sigil | Form | Description |
|------|-------|------|-------------|
| `(#, t)` | HAX | TEXT | Plain UTF-8 text / particle content |
| `(#, T)` | HAX | TABLE | 2D table (see Tables below) |
| `(@, t)` | PAT | TEXT | Neuron / identity reference (`@name`); presentation, not verified authority |
| `(~, t)` | SIG | TEXT | Annotation / label / side-info |
| `(!, e)` | ZAP | ERROR | Typed error — payload is kv struct |
| `(!, c)` | ZAP | COMPONENT | Action button — payload: label `(~, t)` + ref `(#, t)` |
| `(., l)` | DOT | LOG | Structured log line — payload is kv struct |
| `(., p)` | DOT | PROGRESS | Live progress bar — payload is kv struct |
| `(., x)` | DOT | STATUS | End-of-command sentinel — payload is kv struct |
| `(\|, c)` | BAR | COMPONENT | Composition container — payload contains frames |
| `(/, c)` | FAS | COMPONENT | Scope / breadcrumb — payload contains frames |
| `(/, s)` | FAS | STRUCT | Schema row (table header) — payload contains `(~, t)` frames |
| `(:, s)` | COL | STRUCT | Data row — payload contains cell frames |
| `(=, s)` | TIS | STRUCT | Key-value binding — payload: key `(~, t)` + value frame |
| `(?, c)` | WUT | COMPONENT | Input request — payload is kv struct |
| `(?, i)` | WUT | INPUT | User response — payload is raw text |
| `($, t)` | BUC | TEXT | Economic value as text |
| `($, k)` | BUC | TOKEN | Token / ledger entry |
| `(+, t)` | LUS | TEXT | Augmentation / delta as text |

## 4. The kv struct convention

The cyberia catalog encodes key-value bindings using `(=, s)` frames whose
payload is exactly two nested frames:

```
payload = encode[(~, t)[key]] + encode[(sigil, form)[value]]
```

Multiple kv pairs are expressed by nesting them inside a `(|, c)` component
payload:

```
payload-of-component = encode[(=,s)[k1,v1]] + encode[(=,s)[k2,v2]] + …
```

**Reading kv**: decode the payload to get exactly two frames. The first is
the key `(~, t)`, the second is the value (any type). Pairs with fewer than
two inner frames MUST be ignored.

The `read_kv(payload)` helper decodes a payload containing TIS+STRUCT frames
and returns a map from key string to value chunk.

## 5. Structured meta chunks

The catalog's meta chunk types (`(., l)`, `(!, e)`, `(., p)`, `(., x)`) all
use the same convention: their payload is a sequence of `(=, s)` key-value
pairs.

### `(!, e)` error

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `level` | `(#, t)` | no | `"error"` (default), `"warn"` |
| `source` | `(#, t)` | no | Origin (file path, module name) |
| `message` | `(#, t)` | yes | Human-readable description |

### `(., l)` log

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `level` | `(#, t)` | no | `"info"` (default), `"warn"`, `"error"`, `"debug"`, `"trace"` |
| `source` | `(#, t)` | no | Origin |
| `message` | `(#, t)` | yes | Log text |

### `(., p)` progress

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `id` | `(#, t)` | yes | u64 decimal string — used for in-place update |
| `label` | `(#, t)` | no | Description of what is progressing |
| `current` | `(#, t)` | yes | u64 decimal string |
| `total` | `(#, t)` | yes | u64 decimal string (>0) |

Consumers render `current / total` as the fill fraction. A producer SHOULD
send a final progress chunk with `current == total` when done.

For in-place update: a consumer that has rendered a progress chunk with id N
SHOULD update the existing widget rather than appending a new one when a new
`(., p)` chunk arrives with the same id.

### `(., x)` status

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `code` | `(#, t)` | yes | i32 decimal string — process exit code |

A status chunk signals the end of a command's output. Code 0 = success.
Code non-zero = failure. Consumers SHOULD render a visual separator.

## 6. Tables — `(#, T)`

A table chunk's payload contains exactly one schema row followed by zero or
more data rows.

```
(#, T) payload:
  (/, s) [schema]   ← exactly one; payload = concat of (~, t) header frames
  (:, s) [row 1]    ← payload = concat of cell frames (one per column)
  (:, s) [row 2]
  …
```

**Schema row** `(/, s)`: payload contains one `(~, t)` annotation per
column, in order. Column count is defined here.

**Data row** `(:, s)`: payload contains one frame per column, in order.
Cell frames may be any chunk type; text `(#, t)` is the most common.

A consumer MUST tolerate rows with fewer cells than columns (treat missing
cells as empty). A consumer MUST ignore rows with more cells than columns.

```
table_chunk(["name", "size"], [
  [text("Cargo.toml"), text("512 B")],
  [text("src/lib.rs"), text("18 KB")],
])
```

## 7. Composition containers

**`(|, c)` BAR component**: a container whose children are an ordered
sequence of frames to render as a column (vertical stack). Payload contains
nested frames.

**`(/, c)` FAS scope**: a breadcrumb or path container. Same encoding as
BAR component; distinguished by sigil for different rendering (path-style
display).

Children of a component can be any chunk type, including other components
(arbitrary depth).

## 8. Action — `(!, c)`

An action button. Payload contains:

1. `(~, t)` — button label
2. `(#, t)` — target reference (command or URI to invoke on click)

Both frames are required. Order matters.

## 9. Input request `(?, c)` and response `(?, i)`

A producer emits `(?, c)` to request input. Payload is kv:

| Key | Description |
|-----|-------------|
| `prompt` | Text to display to the user |
| `type` | `"text"` (default), `"password"`, `"confirm"` |

The consumer sends a `(?, i)` frame with the raw user input as payload text.

## 10. Catalog-specific conformance

Producers in the cyberia catalog MUST:

- Place exactly one `(~, t)` key frame and one value frame in the payload
  of every `(=, s)` kv pair, key first.
- Place the schema row `(/, s)` before all data rows `(:, s)` in `(#, T)`
  tables.
- Encode progress `id`, `current`, and `total` as decimal strings.
- Encode status `code` as a decimal integer string.

Producers SHOULD:

- Emit a `(., x)` status chunk at the end of each logical command.
- Use progress chunk ids consistently — always increasing, never reused
  within a session.

Consumers MUST:

- Handle `(=, s)` pairs with fewer than two inner frames by ignoring them.
- Handle tables with rows having fewer cells than declared columns by
  treating missing cells as empty.
- Not interpret chunk ids as globally unique — they are scoped to a single
  session or stream.

Consumers SHOULD:

- Render `(., x)` status as a visual separator between command outputs.
- Perform in-place update for `(., p)` progress chunks with a matching id.
- Display `(!, e)` errors prominently, with the `level` field affecting
  visual weight.
- Fall back to raw text display for unknown `(sigil, form)` pairs rather
  than hiding the chunk.

---

## Migration target

The catalog content above belongs in a catalog spec, not in TADE. The
likely target structure once moved:

```
prysm/spec/
├── 0-ontology.md       # the cyberia worldview: atoms (modalities) + molecules
├── 1-atoms.md          # sigils as semantic roles; content + substrate families
├── 2-molecules.md      # error, log, progress, status, table, action, input, …
├── 3-conventions.md    # kv struct, nested payloads, payload schemas
└── 4-conformance.md    # catalog-level producer/consumer requirements
```

When the move happens, this file is deleted from TADE and the TADE spec
references the catalog by name (`urn:cyberia:prysm:1` or similar).
