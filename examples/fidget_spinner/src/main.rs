use fidget_spinner::build_design;
use matter_compiler::compile_prototype;

fn main() {
    let design = build_design();
    let compiled = compile_prototype(&design).expect("example design compiles");

    println!("Example: {}", design.name);
    println!(
        "Target: {} {} via {:?}",
        compiled.plan.machine.manufacturer, compiled.plan.machine.model, compiled.plan.process,
    );
    println!("Material: {:?}", compiled.plan.material);
    println!("Validated parts: {}", compiled.report.validated_part_count);
    println!("Executed passes: {:?}", compiled.report.executed_passes);
    println!(
        "Compiled graph: {} nodes, {} edges",
        compiled.node_count(),
        compiled.edge_count()
    );
}
