use fidget_spinner::build_design;
use matter_compiler::{
    PrototypeArtifactFormat, PrototypeConnectionKind, PrototypeLengthUnit, PrototypePass,
    PrototypeProfile, compile_lowered_prototype, compile_prototype,
};
use matter_context::Material;

#[test]
fn compiles_the_sdk_fidget_spinner_example_end_to_end() {
    let design = build_design();

    let authored = compile_prototype(&design).expect("authored design compiles");
    let lowered = compile_lowered_prototype(design.lower_to_ir().expect("design lowers"))
        .expect("lowered design compiles");

    assert_eq!(authored, lowered);
    assert_eq!(authored.profile, PrototypeProfile::PlaOnP1sSingleProcess);
    assert_eq!(authored.design_name, "fidget_spinner");
    assert_eq!(authored.plan.material, Material::Pla);
    assert_eq!(authored.artifacts.len(), 2);
    assert_eq!(authored.artifacts[0].name, "body");
    assert_eq!(authored.artifacts[0].format, PrototypeArtifactFormat::Stl);
    assert_eq!(
        authored.artifacts[0].mesh.units,
        PrototypeLengthUnit::Millimetres
    );
    assert_eq!(authored.artifacts[0].mesh.triangles.len(), 144);
    assert_eq!(authored.artifacts[1].name, "cap");
    assert_eq!(authored.artifacts[1].mesh.triangles.len(), 96);
    assert_eq!(authored.artifacts[1].mesh.triangles[0].vertices[1].x, 10.0);
    assert_eq!(authored.artifacts[1].mesh.triangles[0].vertices[1].z, 4.0);

    let body_stl = authored
        .stl_artifact("body")
        .expect("body STL export is available");
    let cap_stl = authored
        .stl_artifact("cap")
        .expect("cap STL export is available");

    assert_eq!(
        body_stl,
        lowered
            .stl_artifact("body")
            .expect("lowered body STL exists")
    );
    assert!(body_stl.starts_with(concat!(
        "solid body\n",
        "  facet normal 0.000000 0.000000 1.000000\n",
        "    outer loop\n",
        "      vertex 0.000000 0.000000 8.000000\n",
        "      vertex 30.000000 0.000000 8.000000\n",
        "      vertex 26.971418 4.755789 8.000000\n",
        "    endloop\n",
        "  endfacet\n",
    )));
    assert!(body_stl.ends_with("endsolid body\n"));
    assert_eq!(
        body_stl.matches("\n  facet normal ").count(),
        authored.artifacts[0].mesh.triangles.len()
    );
    assert!(cap_stl.starts_with("solid cap\n"));
    assert_eq!(authored.stl_artifact("missing_part"), None);

    assert_eq!(authored.plan.parts.len(), 2);
    assert_eq!(authored.plan.connections.len(), 1);
    assert_eq!(
        authored.plan.connections[0].kind,
        PrototypeConnectionKind::Fixed
    );
    assert_eq!(authored.report.validated_dimensions.len(), 3);
    assert_eq!(
        authored.report.executed_passes,
        vec![
            PrototypePass::ValidatePrototypeContext,
            PrototypePass::ValidateLoweredGraph,
            PrototypePass::ValidateBuildEnvelope,
            PrototypePass::AssembleSingleProcessPlan,
            PrototypePass::SynthesizeArtifacts,
        ]
    );
}
