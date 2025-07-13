fn main() {
    uniffi::generate_scaffolding("src/rem_core.udl").unwrap();
    
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/rem_core.udl");
}