use std::{env, fs, path::PathBuf};

fn main() {
    let source = PathBuf::from("src/kallyup_core.klc");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = fs::read_to_string(source).expect("read Kallyup KLC core");
    let syntax = kalcite_syntax::parse(&text).expect("parse Kallyup KLC core");
    let hir = kalcite_hir::lower(&syntax).expect("lower Kallyup KLC core");
    let mir = kalcite_mir::lower(&hir);
    let output = kalcite_backend_rust::emit_library(&mir, "").expect("emit Kallyup KLC core");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("kallyup_core.rs"),
        output,
    )
    .unwrap();
}
