use vergen::EmitBuilder;

fn main() {
    // Generate the default 'cargo:' instruction output
    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_rustc()
        .emit()
        .unwrap();
}
