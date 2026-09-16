---
title: tade
tags: cyber, soft3, tade
alias: tade, tape, TAPE, TADE, cyber-tape, typed annotated data exchange
icon: "📼"
crystal-type: entity
crystal-domain: cyber
---
# tade

TADE — Typed Annotated Data Exchange. a byte-stream framing protocol: every frame is `marker · type · size · data`, self-contained and independently decodable; the type byte annotates the data with its kind, and dialects declared on the stream say what the kinds mean.

what a frame carries is data — the bytes of a [[file]], a chunk of one, a status, a thought, a tool call. identity is never on the wire as a primitive; a [[particle]] is computed from the bytes at either end, or travels as 32 bytes of data under a dialect that says so.

the name was tape, and it expanded two different ways in two places — Typed Atomic Particle Exchange in the spec, Typed Annotated Payload Exchange in the crate. renamed 2026-09-16 so every word is true: typed, annotated, data.

- [spec/](spec/0-overview.md) — the framing, wire format, stream control, catalog protocol, conformance, transport bindings, security
- [whitepaper/](whitepaper/whitepaper.md) — why
- [impl/rust/](impl/rust/) — the `tade` crate

see [[soft3]] for the stack · [[prysm]] for the UI dialect that rides on it · [[radio]] for the transport underneath
