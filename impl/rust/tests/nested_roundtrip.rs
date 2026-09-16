//! The shape com builds for a nushell list: one struct chunk whose payload is
//! N struct chunks, each of which is a (label, value) pair.

#[test]
fn nested_roundtrip_keeps_every_chunk() {
    let chunks: Vec<tade::Chunk> =
        (0..60).map(|i| tade::Chunk::text(&format!("row {i}"))).collect();
    let payload = tade::encode_nested(&chunks);
    assert_eq!(tade::decode_nested(&payload).len(), 60);
}

#[test]
fn nested_roundtrip_survives_two_levels() {
    let rows: Vec<tade::Chunk> = (0..60)
        .map(|i| {
            let pair = tade::encode_nested(&[
                tade::Chunk::annotation(&i.to_string()),
                tade::Chunk::text(&format!("value {i}")),
            ]);
            tade::Chunk::new(tade::sigil::COL, tade::render::STRUCT, pair)
        })
        .collect();
    let tree = tade::Chunk::new(
        tade::sigil::FAS,
        tade::render::STRUCT,
        tade::encode_nested(&rows),
    );

    let back = tade::decode_nested(&tree.payload);
    assert_eq!(back.len(), 60, "outer decoded {} of 60", back.len());
    for (i, row) in back.iter().enumerate() {
        let inner = tade::decode_nested(&row.payload);
        assert_eq!(inner.len(), 2, "row {i} decoded {} of 2", inner.len());
    }
}
