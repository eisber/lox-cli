use lox_sim::{engine::SimEngine, parser::parse_bytes};

#[test]
fn test_nfc_edge_propagation() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<LoxoneConfig>
<C Type="Program" U="p1" Title="Program">
<C Type="Page" U="pg1" Title="Page">
<C Type="Place" U="pl1" Title="Room1">
  <C Type="NfcCodeTouch" U="nfc1" Title="NFC">
    <Co K="Disable" U="nfc1-disable"/>
    <Co K="Lr" U="nfc1-lr"/>
    <Co K="Lg" U="nfc1-lg"/>
    <Co K="Lb" U="nfc1-lb"/>
    <Co K="Lw" U="nfc1-lw"/>
    <Co K="Q1" U="nfc1-q1"/>
    <Co K="Q2" U="nfc1-q2"/>
    <Co K="TQ" U="nfc1-tq"/>
    <Co K="TQU" U="nfc1-tqu"/>
    <Co K="TQo" U="nfc1-tqo"/>
    <Co K="TQt" U="nfc1-tqt"/>
    <Co K="Qd" U="nfc1-qd"/>
    <Co K="Qa" U="nfc1-qa"/>
    <Co K="Qn" U="nfc1-qn"/>
    <Co K="Be" U="nfc1-be"/>
  </C>
  <C Type="EdgeDetection" U="ed1" Title="Erkennung">
    <Co K="Input" U="ed1-i">
      <In Input="NFC.Q1" FLG="2"/>
    </Co>
    <Co K="Edge" U="ed1-e"/>
    <Co K="RisingEdge" U="ed1-r"/>
    <Co K="FallingEdge" U="ed1-f"/>
  </C>
</C>
</C>
</C>
</LoxoneConfig>"#;

    let graph = parse_bytes(xml.as_bytes()).expect("parse failed");

    // Print block info
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        eprintln!("Block {} '{}' type={}", bid, info.name, info.block_type);
        for &cid in &info.inputs {
            let c = graph.connector(cid);
            let src = graph.input_source_of(cid);
            eprintln!("  input  cid={} key={} source={:?}", cid, c.key, src);
        }
        for &cid in &info.outputs {
            let c = graph.connector(cid);
            eprintln!("  output cid={} key={}", cid, c.key);
        }
    }

    let mut engine = SimEngine::new(graph.clone());

    let ok = engine.set_input("NFC.Q1", 1.0);
    eprintln!("\nset_input('NFC.Q1', 1.0) -> {}", ok);
    assert!(ok, "set_input NFC.Q1 should succeed");

    // Tick
    engine.tick(0.1);

    // Print all signals
    eprintln!("\nAfter tick 1:");
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        for &cid in info.inputs.iter().chain(info.outputs.iter()) {
            let c = graph.connector(cid);
            let sig = engine.signal(cid);
            if sig != 0.0 {
                eprintln!("  signal[{}] {}.{} = {}", cid, info.name, c.key, sig);
            }
        }
    }

    let edge = engine.get_output("Erkennung.Edge");
    let rising = engine.get_output("Erkennung.RisingEdge");
    eprintln!("Edge={}, Rising={}", edge, rising);

    assert!(edge > 0.5, "Erkennung.Edge should be >0.5, got {}", edge);
}
