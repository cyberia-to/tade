# TAPE — Transport Bindings

TAPE is transport-agnostic. A TAPE stream is any ordered sequence of bytes
containing TAPE frames. This document specifies how TAPE SHOULD be carried
over common transports.

## stdio (stdin / stdout)

The simplest binding. A producer writes TAPE frames to stdout; a consumer
reads from stdin. No handshake, no framing beyond TAPE itself.

**Producer**: write frames directly to stdout with no buffering between frames.
Flush after each logical unit (e.g. after each command's output is complete).

**Consumer**: read from stdin in a loop, feed bytes into a TAPE reader,
process frames as they arrive.

**Use cases**: CLI tools piping typed output, shell-to-terminal communication,
subprocess output capture.

```
producer → stdout → pipe → consumer's stdin
```

TAPE frames and plain text output SHOULD NOT be mixed in the same stream
unless the consumer is known to handle raw bytes before the first `0x1F`.
If mixing is unavoidable, put plain text first (the TAPE decoder discards
bytes before the first frame marker).

## TCP

TAPE rides on a TCP stream with no additional framing. The TCP connection
provides ordered, reliable delivery.

**Connection**: open a TCP connection. No handshake frame is required.
**Sending**: write TAPE frames directly to the TCP socket.
**Receiving**: feed incoming bytes into a TAPE reader; process frames as complete.
**Reconnection**: after reconnect, both sides MUST start fresh — there is no
sequence numbering or resume mechanism in TAPE.

**Port**: no standard TAPE port is assigned. Applications choose their
own port or negotiate via a higher-level protocol.

## WebSocket

Each WebSocket message carries one or more TAPE frames as a binary message
(`opcode=0x02`). Text messages MUST be ignored by a TAPE consumer.

**Sending**: write one or more TAPE frames to a single binary WebSocket message.
There is no requirement that a message contain exactly one frame.

**Receiving**: concatenate binary message payloads and feed into a standard
TAPE reader. Frame boundaries do not align with WebSocket message boundaries.

**Why binary messages**: TAPE frames contain arbitrary bytes; text messages
require valid UTF-8 which TAPE payloads do not guarantee.

## HTTP response body

A streaming HTTP response can carry a TAPE stream. This is useful for
server-sent structured output (e.g. AI agent responses).

**Request**: any method. The request body MAY contain TAPE frames (e.g. for
input chunks in a bidirectional exchange).

**Response headers**:
```
Content-Type: application/x-tape
Transfer-Encoding: chunked   (for streaming)
```

**Response body**: a sequence of TAPE frames, flushed incrementally as the
server produces output.

**End of stream**: when the HTTP response body ends, the TAPE stream ends.
The server SHOULD emit a `(., x)` status chunk as the last frame.

**Content-Type**: `application/x-tape` is the RECOMMENDED content type.
Implementations MAY use `application/octet-stream` as a fallback.

## Unix domain socket

Identical to TCP. Substitute a Unix socket path for the TCP host:port.
Suitable for local IPC between processes on the same machine.

## File

A TAPE file is a regular file containing TAPE frames. No additional file
header or magic bytes are required — the first frame's `0x1F` marker serves
as implicit identification.

**Recommended extension**: `.tape`

**Reading**: open the file, feed all bytes into a TAPE reader, process frames.
**Writing**: append frames to the file. No locking mechanism is specified.

## Future: QUIC

QUIC streams are ordered and reliable within a stream, making them a natural
fit for TAPE. Each QUIC stream MAY carry an independent TAPE stream (useful for
multiplexing multiple terminal sessions). Specification is not yet defined.

## Binding comparison

| Transport | Ordering | Reliability | Bidirectional | Multiplexing |
|-----------|----------|-------------|---------------|--------------|
| stdio | yes | yes (OS pipe) | stdin+stdout | no |
| TCP | yes | yes | yes | no |
| WebSocket | yes | yes | yes | no |
| HTTP body | yes | yes | request+response | no |
| Unix socket | yes | yes | yes | no |
| File | yes | yes (durable) | no | no |
| QUIC stream | yes | yes | yes | yes (future) |
