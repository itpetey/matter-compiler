use fidget_spinner::build_design;
use matter_compiler::{PrototypePass, PrototypeProfile, compile_prototype};
use matter_context::Material;
use matter_ir::{EdgeKind, NodeKind, Value};
use matter_sdk::AuthorKind;

#[test]
fn fidget_spinner_example_validates_and_compiles_end_to_end() {
    let design = build_design();

    assert_eq!(design.validate(), Ok(()));

    let lowered = design.lower_to_ir().expect("example lowers successfully");
    let compiled = compile_prototype(&design).expect("example compiles successfully");
    let graph = &lowered.graph;

    assert_eq!(lowered.manufacturing.target.machine.model, "P1S");
    assert_eq!(compiled.profile, PrototypeProfile::PlaOnP1sSingleProcess);
    assert_eq!(compiled.plan.material, Material::Pla);
    assert_eq!(compiled.artifacts.len(), 2);
    assert_eq!(compiled.artifacts[0].name, "body");
    assert_eq!(compiled.artifacts[1].name, "cap");
    let body_stl = compiled
        .stl_artifact("body")
        .expect("spinner body STL is available");
    let cap_stl = compiled
        .stl_artifact("cap")
        .expect("spinner cap STL is available");

    assert!(body_stl.starts_with("solid body\n"));
    assert!(body_stl.ends_with("endsolid body\n"));
    assert!(cap_stl.starts_with(concat!(
        "solid cap\n",
        "  facet normal 0.000000 0.000000 1.000000\n",
        "    outer loop\n",
        "      vertex 0.000000 0.000000 4.000000\n",
        "      vertex 10.000000 0.000000 4.000000\n",
        "      vertex 9.659258 2.588190 4.000000\n",
        "    endloop\n",
        "  endfacet\n",
    )));
    assert_eq!(
        body_stl.matches("\n  facet normal ").count(),
        compiled.artifacts[0].mesh.triangles.len()
    );
    assert_eq!(
        cap_stl.matches("\n  facet normal ").count(),
        compiled.artifacts[1].mesh.triangles.len()
    );
    assert_eq!(compiled.stl_artifact("missing_part"), None);
    assert_eq!(compiled.report.validated_part_count, 2);
    assert_eq!(compiled.report.validated_connection_count, 1);
    assert_eq!(compiled.report.validated_dimensions.len(), 3);
    assert_eq!(
        compiled.report.executed_passes,
        vec![
            PrototypePass::ValidatePrototypeContext,
            PrototypePass::ValidateLoweredGraph,
            PrototypePass::ValidateBuildEnvelope,
            PrototypePass::AssembleSingleProcessPlan,
            PrototypePass::SynthesizeArtifacts,
        ]
    );
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
        Value::String("examples/fidget_spinner/src/lib.rs".into())
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
    assert_eq!(compiled.node_count(), graph.nodes().len());
    assert_eq!(compiled.edge_count(), graph.edges().len());
}
