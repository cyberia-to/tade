# TADE — Security Considerations

## Scope

TADE is a framing and typing protocol only. It provides no cryptographic
services. The following are explicitly out of scope:

- Confidentiality (encryption)
- Integrity (MACs, signatures)
- Authentication (who sent this stream)
- Authorization (what the receiver is allowed to do with it)
- Replay prevention
- Forward secrecy

Applications that need any of the above MUST use an appropriate transport-layer
security mechanism (TLS, SSH, WireGuard, etc.) beneath TADE.

## Recommended deployment

| Scenario | Recommendation |
|----------|----------------|
| Local stdio (same process) | No additional security needed |
| Local Unix socket | Filesystem permissions provide access control |
| LAN / trusted network | TLS optional; consider network perimeter |
| Internet-facing | TADE over TLS (e.g. WSS, HTTPS, TLS-wrapped TCP) |
| AI agent communication | TLS + application-level authentication |

## Denial-of-service considerations

Data size: a malicious producer can declare an arbitrarily large data
length in the size field. Consumers SHOULD enforce a maximum frame size.
The RECOMMENDED default limit is 64 MiB per frame. Implementations MAY make
this configurable.

Nesting depth: deeply nested frames can cause stack overflows in
recursive decoders. Consumers SHOULD enforce a maximum nesting depth.
The RECOMMENDED default is 64 levels.

Infinite streams: a TADE stream has no built-in length or end marker.
Consumers reading from unbounded sources SHOULD implement timeouts or byte
limits.

Size varint overflow: a varint longer than 10 bytes or encoding a value larger
than 2^63 − 1 MUST be treated as a malformed frame. The decoder MUST advance
past it by scanning for the next `0x1F`.

## Input sanitisation

TADE data fields are opaque bytes. Consumers that render data as text MUST
sanitise for injection vulnerabilities appropriate to their rendering context:

- HTML rendering: escape `<`, `>`, `&`, `"` in text data
- Terminal rendering: strip or escape ANSI escape sequences in text-type frames
- SQL: never interpolate TADE data directly into queries
- Shell: never pass TADE data as shell arguments without quoting

## Forward compatibility and unknown types

The requirement that consumers skip unknown type bytes (rather than
failing) creates a potential for silent data loss if a producer sends a
security-relevant frame type not recognised by an older consumer. Applications
that depend on consumers seeing certain frame types MUST use only well-known,
deployed type codes for security-critical data.
