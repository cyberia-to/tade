use crate::{Chunk, sigil, render, decode_nested, read_kv, kv, encode_nested, table_chunk};
use std::collections::HashMap;

// ── Typed molecule data structs ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Text { pub content: String }

#[derive(Debug, Clone)]
pub struct Annotation { pub content: String }

#[derive(Debug, Clone)]
pub struct Neuron { pub name: String }

#[derive(Debug, Clone)]
pub struct Log { pub level: String, pub source: String, pub message: String }

#[derive(Debug, Clone)]
pub struct ErrorMsg { pub level: String, pub source: String, pub message: String }

#[derive(Debug, Clone)]
pub struct Status { pub code: i32 }

#[derive(Debug, Clone)]
pub struct Progress { pub id: u64, pub label: String, pub current: u64, pub total: u64 }

#[derive(Debug, Clone)]
pub struct Action { pub label: String, pub target: String }

#[derive(Debug, Clone)]
pub struct Component { pub children: Vec<Molecule> }

#[derive(Debug, Clone)]
pub struct Scope { pub children: Vec<Molecule> }

/// Table with pre-decoded string cells (matches the v0 render path).
#[derive(Debug, Clone)]
pub struct Table { pub headers: Vec<String>, pub rows: Vec<Vec<String>> }

// ── Molecule enum ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Molecule {
    Text(Text),
    Annotation(Annotation),
    Neuron(Neuron),
    Log(Log),
    Error(ErrorMsg),
    Status(Status),
    Progress(Progress),
    Action(Action),
    Component(Component),
    Scope(Scope),
    Table(Table),
    Unknown { sigil: u8, render: u8, payload: bytes::Bytes },
}

impl Molecule {
    /// Decode a `Chunk` into its typed `Molecule` representation.
    pub fn from_chunk(c: &Chunk) -> Self {
        match (c.sigil, c.render) {
            (sigil::HAX, render::TEXT) =>
                Molecule::Text(Text { content: str_payload(&c.payload) }),

            (sigil::SIG, render::TEXT) =>
                Molecule::Annotation(Annotation { content: str_payload(&c.payload) }),

            (sigil::PAT, render::TEXT) => {
                let raw = str_payload(&c.payload);
                let name = if raw.starts_with('@') { raw } else { format!("@{raw}") };
                Molecule::Neuron(Neuron { name })
            }

            (sigil::DOT, render::LOG) => {
                let m = read_kv(&c.payload);
                Molecule::Log(Log {
                    level:   kv_str(&m, "level",   "info"),
                    source:  kv_str(&m, "source",  ""),
                    message: kv_str(&m, "message", ""),
                })
            }

            (sigil::ZAP, render::ERROR) => {
                let m = read_kv(&c.payload);
                Molecule::Error(ErrorMsg {
                    level:   kv_str(&m, "level",   "error"),
                    source:  kv_str(&m, "source",  ""),
                    message: kv_str(&m, "message", "unknown error"),
                })
            }

            (sigil::DOT, render::STATUS) => {
                let m = read_kv(&c.payload);
                let code = m.get("code")
                    .and_then(|c| String::from_utf8_lossy(&c.payload).parse().ok())
                    .unwrap_or(0);
                Molecule::Status(Status { code })
            }

            (sigil::DOT, render::PROGRESS) => {
                let m = read_kv(&c.payload);
                let u64v = |key: &str| -> u64 {
                    m.get(key)
                        .and_then(|c| String::from_utf8_lossy(&c.payload).parse().ok())
                        .unwrap_or(0)
                };
                Molecule::Progress(Progress {
                    id:      u64v("id"),
                    label:   kv_str(&m, "label",   ""),
                    current: u64v("current"),
                    total:   u64v("total"),
                })
            }

            (sigil::ZAP, render::COMPONENT) => {
                let chunks = decode_nested(&c.payload);
                let mut label  = String::new();
                let mut target = String::new();
                for inner in &chunks {
                    if inner.sigil == sigil::SIG && inner.render == render::TEXT {
                        label = str_payload(&inner.payload);
                    } else if inner.sigil == sigil::HAX && inner.render == render::TEXT {
                        target = str_payload(&inner.payload);
                    }
                }
                if label.is_empty() { label = "action".into(); }
                Molecule::Action(Action { label, target })
            }

            (sigil::BAR, render::COMPONENT) => {
                let children = decode_nested(&c.payload)
                    .iter().map(Molecule::from_chunk).collect();
                Molecule::Component(Component { children })
            }

            (sigil::FAS, render::COMPONENT) => {
                let children = decode_nested(&c.payload)
                    .iter().map(Molecule::from_chunk).collect();
                Molecule::Scope(Scope { children })
            }

            (sigil::HAX, render::TABLE) => {
                let rows = decode_nested(&c.payload);
                let mut headers: Vec<String>        = Vec::new();
                let mut data_rows: Vec<Vec<String>>  = Vec::new();
                for row in &rows {
                    if row.sigil == sigil::FAS && row.render == render::STRUCT {
                        for h in decode_nested(&row.payload) {
                            headers.push(str_payload(&h.payload));
                        }
                    } else if row.sigil == sigil::COL && row.render == render::STRUCT {
                        let cells = decode_nested(&row.payload);
                        data_rows.push(cells.iter().map(|c| str_payload(&c.payload)).collect());
                    }
                }
                Molecule::Table(Table { headers, rows: data_rows })
            }

            _ => Molecule::Unknown { sigil: c.sigil, render: c.render, payload: c.payload.clone() },
        }
    }

    /// Encode back to a `Chunk` (lossless for all typed variants).
    pub fn to_chunk(&self) -> Chunk {
        match self {
            Molecule::Text(t)       => Chunk::text(&t.content),
            Molecule::Annotation(a) => Chunk::annotation(&a.content),
            Molecule::Neuron(n)     => Chunk::new(sigil::PAT, render::TEXT,
                bytes::Bytes::copy_from_slice(n.name.as_bytes())),
            Molecule::Log(l)        => Chunk::log(&l.level, &l.source, &l.message),
            Molecule::Error(e)      => {
                let payload = encode_nested(&[
                    kv("level",   Chunk::text(&e.level)),
                    kv("source",  Chunk::text(&e.source)),
                    kv("message", Chunk::text(&e.message)),
                ]);
                Chunk::new(sigil::ZAP, render::ERROR, payload)
            }
            Molecule::Status(s)     => Chunk::status(s.code),
            Molecule::Progress(p)   => Chunk::progress(p.id, &p.label, p.current, p.total),
            Molecule::Action(a)     => {
                let payload = encode_nested(&[
                    Chunk::annotation(&a.label),
                    Chunk::text(&a.target),
                ]);
                Chunk::new(sigil::ZAP, render::COMPONENT, payload)
            }
            Molecule::Component(c)  => {
                let payload = encode_nested(&c.children.iter()
                    .map(|m| m.to_chunk()).collect::<Vec<_>>());
                Chunk::new(sigil::BAR, render::COMPONENT, payload)
            }
            Molecule::Scope(s)      => {
                let payload = encode_nested(&s.children.iter()
                    .map(|m| m.to_chunk()).collect::<Vec<_>>());
                Chunk::new(sigil::FAS, render::COMPONENT, payload)
            }
            Molecule::Table(t)      => {
                let header_refs: Vec<&str> = t.headers.iter().map(|s| s.as_str()).collect();
                let rows: Vec<Vec<Chunk>> = t.rows.iter()
                    .map(|row| row.iter().map(|s| Chunk::text(s)).collect())
                    .collect();
                table_chunk(&header_refs, rows)
            }
            Molecule::Unknown { sigil, render, payload } =>
                Chunk::new(*sigil, *render, payload.clone()),
        }
    }
}

// ── private helpers ───────────────────────────────────────────────────────────

fn str_payload(payload: &[u8]) -> String {
    String::from_utf8_lossy(payload).into_owned()
}

fn kv_str(m: &HashMap<String, Chunk>, key: &str, default: &str) -> String {
    m.get(key)
        .map(|c| String::from_utf8_lossy(&c.payload).into_owned())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(m: &Molecule) -> Molecule {
        Molecule::from_chunk(&m.to_chunk())
    }

    #[test]
    fn text_round_trip() {
        let m = Molecule::from_chunk(&Chunk::text("hello"));
        match &m {
            Molecule::Text(t) => assert_eq!(t.content, "hello"),
            other => panic!("expected Text, got {other:?}"),
        }
        match round_trip(&m) {
            Molecule::Text(t) => assert_eq!(t.content, "hello"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn annotation_round_trip() {
        match Molecule::from_chunk(&Chunk::annotation("note")) {
            Molecule::Annotation(a) => assert_eq!(a.content, "note"),
            other => panic!("expected Annotation, got {other:?}"),
        }
    }

    #[test]
    fn neuron_adds_missing_at_prefix() {
        let c = Chunk::new(sigil::PAT, render::TEXT, bytes::Bytes::from_static(b"alice"));
        match Molecule::from_chunk(&c) {
            Molecule::Neuron(n) => assert_eq!(n.name, "@alice"),
            other => panic!("expected Neuron, got {other:?}"),
        }
    }

    #[test]
    fn neuron_keeps_existing_at_prefix() {
        let c = Chunk::new(sigil::PAT, render::TEXT, bytes::Bytes::from_static(b"@alice"));
        match Molecule::from_chunk(&c) {
            Molecule::Neuron(n) => assert_eq!(n.name, "@alice"),
            other => panic!("expected Neuron, got {other:?}"),
        }
    }

    #[test]
    fn neuron_round_trip_preserves_at_prefix() {
        let m = Molecule::Neuron(Neuron { name: "@alice".into() });
        match round_trip(&m) {
            Molecule::Neuron(n) => assert_eq!(n.name, "@alice"),
            other => panic!("expected Neuron, got {other:?}"),
        }
    }

    #[test]
    fn log_round_trip() {
        let m = Molecule::from_chunk(&Chunk::log("warn", "core", "boom"));
        match &m {
            Molecule::Log(l) => {
                assert_eq!(l.level, "warn");
                assert_eq!(l.source, "core");
                assert_eq!(l.message, "boom");
            }
            other => panic!("expected Log, got {other:?}"),
        }
        match round_trip(&m) {
            Molecule::Log(l) => {
                assert_eq!(l.level, "warn");
                assert_eq!(l.source, "core");
                assert_eq!(l.message, "boom");
            }
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn log_missing_fields_use_defaults() {
        let payload = encode_nested(&[]);
        let c = Chunk::new(sigil::DOT, render::LOG, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Log(l) => {
                assert_eq!(l.level, "info");
                assert_eq!(l.source, "");
                assert_eq!(l.message, "");
            }
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn error_round_trip() {
        let m = Molecule::from_chunk(&Chunk::error("bad input"));
        match &m {
            Molecule::Error(e) => {
                assert_eq!(e.level, "error");
                assert_eq!(e.message, "bad input");
            }
            other => panic!("expected Error, got {other:?}"),
        }
        match round_trip(&m) {
            Molecule::Error(e) => {
                assert_eq!(e.level, "error");
                assert_eq!(e.message, "bad input");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn error_missing_message_uses_default() {
        let payload = encode_nested(&[]);
        let c = Chunk::new(sigil::ZAP, render::ERROR, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Error(e) => assert_eq!(e.message, "unknown error"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn status_round_trip() {
        let m = Molecule::from_chunk(&Chunk::status(42));
        match &m {
            Molecule::Status(s) => assert_eq!(s.code, 42),
            other => panic!("expected Status, got {other:?}"),
        }
        match round_trip(&m) {
            Molecule::Status(s) => assert_eq!(s.code, 42),
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[test]
    fn status_non_numeric_code_defaults_to_zero() {
        let payload = encode_nested(&[kv("code", Chunk::text("not-a-number"))]);
        let c = Chunk::new(sigil::DOT, render::STATUS, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Status(s) => assert_eq!(s.code, 0),
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[test]
    fn progress_round_trip() {
        let m = Molecule::from_chunk(&Chunk::progress(7, "loading", 3, 10));
        match &m {
            Molecule::Progress(p) => {
                assert_eq!(p.id, 7);
                assert_eq!(p.label, "loading");
                assert_eq!(p.current, 3);
                assert_eq!(p.total, 10);
            }
            other => panic!("expected Progress, got {other:?}"),
        }
        match round_trip(&m) {
            Molecule::Progress(p) => {
                assert_eq!(p.id, 7);
                assert_eq!(p.label, "loading");
                assert_eq!(p.current, 3);
                assert_eq!(p.total, 10);
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn progress_non_numeric_fields_default_to_zero() {
        let payload = encode_nested(&[
            kv("id", Chunk::text("nope")),
            kv("current", Chunk::text("nope")),
            kv("total", Chunk::text("nope")),
        ]);
        let c = Chunk::new(sigil::DOT, render::PROGRESS, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Progress(p) => {
                assert_eq!(p.id, 0);
                assert_eq!(p.current, 0);
                assert_eq!(p.total, 0);
                assert_eq!(p.label, "");
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn action_extracts_label_and_target() {
        let payload = encode_nested(&[Chunk::annotation("open"), Chunk::text("/path")]);
        let c = Chunk::new(sigil::ZAP, render::COMPONENT, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Action(a) => {
                assert_eq!(a.label, "open");
                assert_eq!(a.target, "/path");
            }
            other => panic!("expected Action, got {other:?}"),
        }
    }

    #[test]
    fn action_missing_label_defaults_to_action() {
        let payload = encode_nested(&[Chunk::text("/path")]);
        let c = Chunk::new(sigil::ZAP, render::COMPONENT, payload);
        match Molecule::from_chunk(&c) {
            Molecule::Action(a) => {
                assert_eq!(a.label, "action");
                assert_eq!(a.target, "/path");
            }
            other => panic!("expected Action, got {other:?}"),
        }
    }

    #[test]
    fn action_round_trip() {
        let m = Molecule::Action(Action { label: "open".into(), target: "/path".into() });
        match round_trip(&m) {
            Molecule::Action(a) => {
                assert_eq!(a.label, "open");
                assert_eq!(a.target, "/path");
            }
            other => panic!("expected Action, got {other:?}"),
        }
    }

    #[test]
    fn component_round_trip_nests_children() {
        let m = Molecule::Component(Component {
            children: vec![
                Molecule::Text(Text { content: "a".into() }),
                Molecule::Annotation(Annotation { content: "b".into() }),
            ],
        });
        match round_trip(&m) {
            Molecule::Component(c) => {
                assert_eq!(c.children.len(), 2);
                match &c.children[0] {
                    Molecule::Text(t) => assert_eq!(t.content, "a"),
                    other => panic!("expected Text, got {other:?}"),
                }
                match &c.children[1] {
                    Molecule::Annotation(a) => assert_eq!(a.content, "b"),
                    other => panic!("expected Annotation, got {other:?}"),
                }
            }
            other => panic!("expected Component, got {other:?}"),
        }
    }

    #[test]
    fn scope_round_trip_nests_children() {
        let m = Molecule::Scope(Scope {
            children: vec![Molecule::Text(Text { content: "inner".into() })],
        });
        match round_trip(&m) {
            Molecule::Scope(s) => {
                assert_eq!(s.children.len(), 1);
                match &s.children[0] {
                    Molecule::Text(t) => assert_eq!(t.content, "inner"),
                    other => panic!("expected Text, got {other:?}"),
                }
            }
            other => panic!("expected Scope, got {other:?}"),
        }
    }

    #[test]
    fn table_round_trip() {
        let headers = vec!["name".to_string(), "age".to_string()];
        let rows = vec![
            vec!["alice".to_string(), "30".to_string()],
            vec!["bob".to_string(), "25".to_string()],
        ];
        let m = Molecule::Table(Table { headers: headers.clone(), rows: rows.clone() });
        match round_trip(&m) {
            Molecule::Table(t) => {
                assert_eq!(t.headers, headers);
                assert_eq!(t.rows, rows);
            }
            other => panic!("expected Table, got {other:?}"),
        }
    }

    #[test]
    fn table_decodes_from_table_chunk_helper() {
        let chunk = table_chunk(&["a", "b"], vec![vec![Chunk::text("1"), Chunk::text("2")]]);
        match Molecule::from_chunk(&chunk) {
            Molecule::Table(t) => {
                assert_eq!(t.headers, vec!["a".to_string(), "b".to_string()]);
                assert_eq!(t.rows, vec![vec!["1".to_string(), "2".to_string()]]);
            }
            other => panic!("expected Table, got {other:?}"),
        }
    }

    #[test]
    fn unrecognized_sigil_render_pair_falls_back_to_unknown() {
        let payload = bytes::Bytes::from_static(b"raw bytes");
        let c = Chunk::new(sigil::WUT, render::BINARY, payload.clone());
        match Molecule::from_chunk(&c) {
            Molecule::Unknown { sigil, render, payload: p } => {
                assert_eq!(sigil, sigil::WUT);
                assert_eq!(render, render::BINARY);
                assert_eq!(p, payload);
            }
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn unknown_round_trip_preserves_bytes() {
        let payload = bytes::Bytes::from_static(b"opaque");
        let m = Molecule::Unknown { sigil: sigil::WUT, render: render::BINARY, payload: payload.clone() };
        let c = m.to_chunk();
        assert_eq!(c.sigil, sigil::WUT);
        assert_eq!(c.render, render::BINARY);
        assert_eq!(c.payload, payload);
    }
}
