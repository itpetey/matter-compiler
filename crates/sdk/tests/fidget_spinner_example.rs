#[path = "../examples/fidget_spinner.rs"]
mod fidget_spinner;

use matter_ir::{EdgeKind, NodeKind, Value};
use matter_sdk::AuthorKind;

#[test]
fn fidget_spinner_example_validates_and_lowers_end_to_end() {
    let design = fidget_spinner::build_design();

    assert_eq!(design.validate(), Ok(()));

    let lowered = design.lower_to_ir().expect("example lowers successfully");
    let graph = &lowered.graph;

    assert_eq!(lowered.manufacturing.target.machine.model, "P1S");
    assert_eq!(
        lowered.manufacturing.constraints.supported_materials.len(),
        1
    );
    assert_eq!(design.provenance.author_kind, AuthorKind::Human);
    assert_eq!(
        graph.node(graph.root()).expect("root exists").kind,
        NodeKind::System
    );
    assert_eq!(
        graph.metadata()["sdk.source"],
        Value::String("crates/design-sdk/examples/fidget_spinner.rs".into())
    );
    assert!(graph.nodes().values().any(|node| node.name == "spinner"));
    assert!(graph.nodes().values().any(|node| node.name == "body"));
    assert!(graph.nodes().values().any(|node| node.name == "cap"));
    assert!(
        graph
            .edges()
            .values()
            .any(|edge| edge.kind == EdgeKind::Connects)
    );
    assert_eq!(graph.nodes().len(), 11);
    assert_eq!(graph.edges().len(), 12);
}
