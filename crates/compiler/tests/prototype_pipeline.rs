#[path = "../../sdk/examples/fidget_spinner.rs"]
mod fidget_spinner;

use matter_compiler::{
    PrototypeConnectionKind, PrototypePass, PrototypeProfile, compile_lowered_prototype,
    compile_prototype,
};
use matter_context::Material;

#[test]
fn compiles_the_sdk_fidget_spinner_example_end_to_end() {
    let design = fidget_spinner::build_design();

    let authored = compile_prototype(&design).expect("authored design compiles");
    let lowered = compile_lowered_prototype(design.lower_to_ir().expect("design lowers"))
        .expect("lowered design compiles");

    assert_eq!(authored, lowered);
    assert_eq!(authored.profile, PrototypeProfile::PlaOnP1sSingleProcess);
    assert_eq!(authored.design_name, "fidget_spinner");
    assert_eq!(authored.plan.material, Material::Pla);
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
        ]
    );
}
