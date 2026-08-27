//! The shape com builds for a nushell list: one struct chunk whose payload is
//! N struct chunks, each of which is a (label, value) pair.

#[test]
fn nested_roundtrip_keeps_every_chunk() {
    let chunks: Vec<tape::Chunk> =
        (0..60).map(|i| tape::Chunk::text(&format!("row {i}"))).collect();
    let payload = tape::encode_nested(&chunks);
    assert_eq!(tape::decode_nested(&payload).len(), 60);
}

#[test]
fn nested_roundtrip_survives_two_levels() {
    let rows: Vec<tape::Chunk> = (0..60)
        .map(|i| {
            let pair = tape::encode_nested(&[
                tape::Chunk::annotation(&i.to_string()),
                tape::Chunk::text(&format!("value {i}")),
            ]);
            tape::Chunk::new(tape::sigil::COL, tape::render::STRUCT, pair)
        })
        .collect();
    let tree = tape::Chunk::new(
        tape::sigil::FAS,
        tape::render::STRUCT,
        tape::encode_nested(&rows),
    );

    let back = tape::decode_nested(&tree.payload);
    assert_eq!(back.len(), 60, "outer decoded {} of 60", back.len());
    for (i, row) in back.iter().enumerate() {
        let inner = tape::decode_nested(&row.payload);
        assert_eq!(inner.len(), 2, "row {i} decoded {} of 2", inner.len());
    }
}
