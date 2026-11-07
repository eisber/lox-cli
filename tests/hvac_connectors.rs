use std::process::Command;

fn run(args: &[&str]) {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("lox"))
        .args(args)
        .output()
        .expect("run lox");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn newly_created_room_controller_exposes_and_wires_third_source_ports() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("hvac.Loxone");
    std::fs::write(&file, include_str!("ux/hvac/seed.Loxone")).unwrap();
    let path = file.to_str().unwrap();
    run(&[
        "config",
        "add",
        "--type",
        "HeatIRoomController2",
        "--title",
        "Controller",
        "--room",
        "HVAC Lab",
        "--page",
        "HVAC Lab",
        path,
    ]);
    let xml = std::fs::read(&file).unwrap();
    let document = xmltree::Element::parse(xml.as_slice()).unwrap();
    fn controller(element: &xmltree::Element) -> Option<&xmltree::Element> {
        if element.attributes.get("Title").map(String::as_str) == Some("Controller") {
            return Some(element);
        }
        element
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .find_map(controller)
    }
    let block = controller(&document).unwrap();
    for key in ["RtD", "AQh3", "AQc3", "AQhc3"] {
        assert!(
            block
                .children
                .iter()
                .filter_map(xmltree::XMLNode::as_element)
                .any(|connector| connector.name == "Co"
                    && connector.attributes.get("K").map(String::as_str) == Some(key)),
            "new controller omitted connector {key}"
        );
    }
    let output = Command::new(assert_cmd::cargo::cargo_bin!("lox"))
        .args(["config", "connectors", path, "Controller", "-o", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let inspection: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let connectors = inspection["connectors"].as_array().unwrap();
    for (key, direction) in [("RtD", "I"), ("AQh3", "O"), ("AQc3", "O"), ("AQhc3", "O")] {
        assert!(
            connectors
                .iter()
                .any(|connector| connector["key"] == key && connector["direction"] == direction),
            "new connector {key} has the wrong direction"
        );
    }
    for (index, output) in ["AQh3", "AQc3", "AQhc3"].iter().enumerate() {
        let title = format!("Monitor {index}");
        run(&["config", "add", "--type", "StateV", "--title", &title, path]);
        run(&[
            "config",
            "wire-connector",
            path,
            &format!("{title}.I"),
            &format!("Controller.{output}"),
        ]);
    }
}
