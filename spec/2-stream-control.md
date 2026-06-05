# TAPE — Stream Control

TAPE reserves a small namespace for stream-level control: frames that
manage the stream itself, independent of any dialect. Every TAPE consumer
MUST understand the reserved set even if it does not understand the
declared dialect.

## Reserved type byte

TAPE reserves the type byte `*` (0x2A) for stream control. No dialect
may use `*` as a type byte. All `(*, *)` pairs are owned by TAPE.

This keeps dialect vocabulary and stream control fully separated: a
consumer encountering `*` knows the frame is TAPE-defined, regardless of
which dialect is active.

## Reserved frames

The minimum set in this version of TAPE:

| Pair | Name | Data | Description |
|------|------|------|-------------|
| `(*, k)` | Dialect declaration | UTF-8 dialect URN | Announces which dialect the stream uses |
| `(*, c)` | Cancel | UTF-8 reason (optional, may be empty) | Producer or consumer is aborting the stream |
| `(*, h)` | Heartbeat | empty | Keep-alive ping; no semantic content |

These three are the entire TAPE Layer 1 vocabulary.

## `(*, k)` — Dialect declaration

The first frame of a TAPE stream SHOULD be a dialect declaration. Its
data is a UTF-8 dialect identifier (see
[3-catalog-protocol.md](3-catalog-protocol.md) for the identifier
format).

```
(*, k) data: "urn:cyberia:prysm:1"
```

A stream MAY omit the dialect declaration; in that case the consumer
treats the stream as dialect-unknown and processes only stream
control frames, skipping all others.

A stream MAY re-declare its dialect mid-stream by emitting another `(*, k)`
frame. All frames after the re-declaration are interpreted in the new
dialect.

## `(*, c)` — Cancel

Either side of a TAPE exchange may send `(*, c)` to signal abort. The
data, if non-empty, is a UTF-8 reason string for logging or display.

A producer that emits `(*, c)` SHOULD stop emitting further frames. A
consumer that emits `(*, c)` on a writable channel SHOULD expect the
producer to honour it.

`(*, c)` does not close the underlying transport — that is the
transport's responsibility. It signals semantic intent.

## `(*, h)` — Heartbeat

A producer MAY emit `(*, h)` periodically to indicate liveness on a
stream with sparse traffic. Heartbeats have empty data. Consumers
treat them as no-ops.

## Why these and only these

Stream control is what every TAPE consumer needs regardless of dialect:

- Dialect declaration — so the consumer knows how to interpret
  application frames.
- Cancel — so streams have a clean abort signal that survives any
  dialect change.
- Heartbeat — so long-lived streams can detect dead transports
  without parsing application content.

Anything else — progress, error, log, status, content addresses,
capability tokens, neuron identity — belongs in a dialect, not in
TAPE. A consumer can implement TAPE in a few hundred lines and handle
arbitrary streams; semantic richness comes from whichever dialect the
stream declares.

## Future reservations

TAPE may extend the `(*, *)` namespace in future revisions. The
following are reserved for potential future use but not currently
defined:

- Flow control / backpressure signals
- Capability negotiation / feature advertisement
- Stream multiplexing identifiers

Dialects MUST NOT use `*` as a type byte regardless of which `(*, *)` pairs
TAPE currently defines.
