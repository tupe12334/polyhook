use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let schema_path = PathBuf::from(&manifest_dir).join("schema.json");

    println!("cargo:rerun-if-changed={}", schema_path.display());

    let schema_str = fs::read_to_string(&schema_path).unwrap_or_else(|e| {
        panic!(
            "build.rs: cannot read schema.json at {}: {}",
            schema_path.display(),
            e
        )
    });

    let schema: schemars::schema::RootSchema = serde_json::from_str(&schema_str)
        .unwrap_or_else(|e| panic!("build.rs: schema.json is not valid JSON Schema: {e}"));

    let settings = typify::TypeSpaceSettings::default();
    let mut space = typify::TypeSpace::new(&settings);
    space
        .add_root_schema(schema)
        .unwrap_or_else(|e| panic!("build.rs: typify failed: {e:?}"));

    let tokens = space.to_stream();
    let ast = syn::parse2::<syn::File>(tokens)
        .unwrap_or_else(|e| panic!("build.rs: syn parse failed: {e}"));
    let code = prettyplease::unparse(&ast);

    let out_dir = env::var("OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("types.rs");
    fs::write(&out_path, code)
        .unwrap_or_else(|e| panic!("build.rs: cannot write types.rs to OUT_DIR: {e}"));

    Ok(())
}
