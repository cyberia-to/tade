// ---
// tags: tade, rust, cli
// crystal-type: source
// crystal-domain: comp
// ---
//! tade — inspect and mint cyb-stream frames.
//!
//!   tade inspect [file]              decode a tade stream into a frame table
//!   tade sigils                      the 13 sigils (the frame's type)
//!   tade renders                     the 15 render kinds (how to show a payload)
//!   tade make <sigil> <render> <s>   emit one frame to stdout
//!
//! A tade frame is `MARKER(0x1F) · sigil · render · varint(len) · payload`. Every
//! durable log and every gossiped message in the stack is a run of these, so
//! `tade inspect` is the debugger for the wire.

use std::io::{self, IsTerminal, Read, Write};

use tade::{render, sigil, Chunk, ReadResult, Reader};

// ── color ────────────────────────────────────────────────────────────────────

fn tty() -> bool {
    io::stdout().is_terminal()
}
fn paint(code: &str, s: &str) -> String {
    if tty() { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
}
fn dim(s: &str) -> String {
    paint("90", s)
}
fn cyan(s: &str) -> String {
    paint("36", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn bold(s: &str) -> String {
    paint("1", s)
}
fn red(s: &str) -> String {
    paint("31", s)
}

const LOGO: &str = "\
\x1b[31m████████╗ █████╗ ██████╗ ███████╗\x1b[0m
\x1b[33m╚══██╔══╝██╔══██╗██╔══██╗██╔════╝\x1b[0m
\x1b[32m   ██║   ███████║██████╔╝█████╗  \x1b[0m
\x1b[36m   ██║   ██╔══██║██╔═══╝ ██╔══╝  \x1b[0m
\x1b[34m   ██║   ██║  ██║██║     ███████╗\x1b[0m
\x1b[35m   ╚═╝   ╚═╝  ╚═╝╚═╝     ╚══════╝\x1b[0m";

fn banner() {
    if !tty() {
        return;
    }
    println!("{LOGO}");
    println!("{}", paint("37", "    the wire — self-describing frames"));
    println!("{}", dim("\n    MARKER 0x1F · sigil · render · varint · payload\n    13 sigils · 15 renders\n"));
}

fn help() {
    banner();
    let rows = [
        ("inspect [file]", "decode a tade stream into a frame table (stdin if no file)"),
        ("sigils", "the 13 sigils — a frame's type"),
        ("renders", "the 15 render kinds — how to show a payload"),
        ("make <sigil> <render> <s>", "emit one frame to stdout"),
    ];
    let w = rows.iter().map(|(c, _)| c.len()).max().unwrap_or(0);
    println!("{}", dim("commands"));
    for (cmd, desc) in rows {
        println!("  {}   {}", bold(&format!("{cmd:<w$}")), dim(desc));
    }
}

// ── inspect ──────────────────────────────────────────────────────────────────

/// A short, human preview of a payload: text renders as a quoted string, the
/// rest as a hex prefix.
fn preview(c: &Chunk) -> String {
    let textual = matches!(c.render, render::TEXT | render::LOG | render::ERROR | render::INPUT);
    if textual {
        let s: String =
            String::from_utf8_lossy(&c.payload).chars().take(56).collect::<String>().replace('\n', "⏎");
        format!("\"{s}\"")
    } else if c.payload.is_empty() {
        "·".into()
    } else {
        let hex: String = c.payload.iter().take(12).map(|b| format!("{b:02x}")).collect();
        format!("{hex}{}", if c.payload.len() > 12 { "…" } else { "" })
    }
}

fn cmd_inspect(path: Option<&str>) {
    let mut bytes = Vec::new();
    let read = match path {
        Some(p) => std::fs::File::open(p).and_then(|mut f| f.read_to_end(&mut bytes)),
        None => io::stdin().read_to_end(&mut bytes),
    };
    if let Err(e) = read {
        eprintln!("  {}: {}", red("error"), e);
        std::process::exit(1);
    }

    let mut reader = Reader::new();
    reader.feed(&bytes);
    println!(
        "  {}   {}  {}  {}  {}",
        dim("#"),
        dim(&format!("{:<10}", "sigil")),
        dim(&format!("{:<12}", "render")),
        dim(&format!("{:>5}", "bytes")),
        dim("payload"),
    );
    let mut n = 0u64;
    loop {
        match reader.next_chunk() {
            ReadResult::Chunk(c) => {
                let sig = format!("{} {}", c.sigil as char, sigil::name(c.sigil));
                let ren = format!("{} {}", c.render as char, render::name(c.render));
                println!(
                    "  {:>2}   {}  {}  {}  {}",
                    yellow(&n.to_string()),
                    cyan(&format!("{sig:<10}")),
                    green(&format!("{ren:<12}")),
                    &format!("{:>5}", c.payload.len()),
                    dim(&preview(&c)),
                );
                n += 1;
            }
            ReadResult::Pending | ReadResult::Eof => break,
        }
    }
    if n == 0 {
        println!("  {}", dim("(no frames)"));
    } else {
        println!("  {}", dim(&format!("{n} frame(s), {} bytes", bytes.len())));
    }
}

// ── reference tables ─────────────────────────────────────────────────────────

fn cmd_sigils() {
    println!("{}", dim("sigils — a frame's type"));
    for b in sigil::ALL {
        println!("  {}  {}", cyan(&format!("{}  0x{:02X}", b as char, b)), bold(sigil::name(b)));
    }
}

fn cmd_renders() {
    println!("{}", dim("renders — how to show a payload"));
    for b in render::ALL {
        println!("  {}  {}", cyan(&format!("{}  0x{:02X}", b as char, b)), bold(render::name(b)));
    }
}

// ── make ─────────────────────────────────────────────────────────────────────

/// Resolve a sigil from a single char or a name (`!` or `zap`).
fn sigil_byte(s: &str) -> Option<u8> {
    if s.len() == 1 && sigil::is_valid(s.as_bytes()[0]) {
        return Some(s.as_bytes()[0]);
    }
    sigil::ALL.into_iter().find(|&b| sigil::name(b).eq_ignore_ascii_case(s))
}

/// Resolve a render from a single char or a name (`t` or `text`).
fn render_byte(s: &str) -> Option<u8> {
    if s.len() == 1 && render::is_valid(s.as_bytes()[0]) {
        return Some(s.as_bytes()[0]);
    }
    render::ALL.into_iter().find(|&b| render::name(b).eq_ignore_ascii_case(s))
}

fn cmd_make(args: &[String]) {
    let (Some(s), Some(r)) = (args.first(), args.get(1)) else {
        eprintln!("  {}: tade make <sigil> <render> [payload]", dim("usage"));
        std::process::exit(2);
    };
    let Some(sig) = sigil_byte(s) else {
        eprintln!("  {}: unknown sigil '{s}' (try `tade sigils`)", red("error"));
        std::process::exit(2);
    };
    let Some(ren) = render_byte(r) else {
        eprintln!("  {}: unknown render '{r}' (try `tade renders`)", red("error"));
        std::process::exit(2);
    };
    let payload = args.get(2).cloned().unwrap_or_default();
    let frame = Chunk::new(sig, ren, payload.into_bytes().into()).encode();
    io::stdout().write_all(&frame).ok();
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("inspect" | "read" | "cat") => cmd_inspect(args.get(1).map(String::as_str)),
        Some("sigils") => cmd_sigils(),
        Some("renders") => cmd_renders(),
        Some("make" | "encode") => cmd_make(&args[1..]),
        Some("help" | "--help" | "-h") | None => help(),
        Some(other) => {
            eprintln!("  {}: {other}  (try: tade help)", dim("unknown"));
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(sigil: u8, render: u8, payload: &str) -> Chunk {
        Chunk::new(sigil, render, bytes::Bytes::from(payload.as_bytes().to_vec()))
    }

    // ── sigil_byte ───────────────────────────────────────────────────────

    #[test]
    fn sigil_byte_single_char() {
        assert_eq!(sigil_byte("!"), Some(sigil::ZAP));
        assert_eq!(sigil_byte("#"), Some(sigil::HAX));
    }

    #[test]
    fn sigil_byte_by_name() {
        assert_eq!(sigil_byte("zap"), Some(sigil::ZAP));
        assert_eq!(sigil_byte("ZAP"), Some(sigil::ZAP));
        assert_eq!(sigil_byte("Zap"), Some(sigil::ZAP));
    }

    #[test]
    fn sigil_byte_rejects_unknown() {
        assert_eq!(sigil_byte("a"), None); // single char, not a sigil byte
        assert_eq!(sigil_byte("nonsense"), None);
        assert_eq!(sigil_byte(""), None);
    }

    // ── render_byte ──────────────────────────────────────────────────────

    #[test]
    fn render_byte_single_char() {
        assert_eq!(render_byte("t"), Some(render::TEXT));
        assert_eq!(render_byte("T"), Some(render::TABLE)); // case-sensitive single-char form
    }

    #[test]
    fn render_byte_by_name() {
        assert_eq!(render_byte("text"), Some(render::TEXT));
        assert_eq!(render_byte("TEXT"), Some(render::TEXT));
        assert_eq!(render_byte("table"), Some(render::TABLE));
    }

    #[test]
    fn render_byte_rejects_unknown() {
        assert_eq!(render_byte("q"), None);
        assert_eq!(render_byte("nonsense"), None);
        assert_eq!(render_byte(""), None);
    }

    // ── preview ──────────────────────────────────────────────────────────

    #[test]
    fn preview_text_is_quoted() {
        let c = chunk(sigil::HAX, render::TEXT, "hello");
        assert_eq!(preview(&c), "\"hello\"");
    }

    #[test]
    fn preview_log_and_error_and_input_are_textual() {
        for r in [render::LOG, render::ERROR, render::INPUT] {
            let c = chunk(sigil::HAX, r, "line");
            assert_eq!(preview(&c), "\"line\"");
        }
    }

    #[test]
    fn preview_text_replaces_newlines() {
        let c = chunk(sigil::HAX, render::TEXT, "a\nb\nc");
        assert_eq!(preview(&c), "\"a⏎b⏎c\"");
    }

    #[test]
    fn preview_text_truncates_at_56_chars() {
        let long = "x".repeat(100);
        let c = chunk(sigil::HAX, render::TEXT, &long);
        let out = preview(&c);
        // quotes + 56 chars
        assert_eq!(out.chars().count(), 58);
        assert!(out.starts_with('"') && out.ends_with('"'));
    }

    #[test]
    fn preview_empty_payload_is_dot() {
        let c = chunk(sigil::HAX, render::BINARY, "");
        assert_eq!(preview(&c), "·");
    }

    #[test]
    fn preview_non_textual_is_hex_prefix() {
        let c = Chunk::new(sigil::HAX, render::BINARY, bytes::Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(preview(&c), "deadbeef");
    }

    #[test]
    fn preview_non_textual_truncates_past_12_bytes() {
        let payload: Vec<u8> = (0u8..20).collect();
        let c = Chunk::new(sigil::HAX, render::BINARY, bytes::Bytes::from(payload));
        let out = preview(&c);
        assert!(out.ends_with('…'));
        assert_eq!(out.len(), 12 * 2 + '…'.len_utf8());
    }
}
