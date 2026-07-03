use include_dir::{include_dir, Dir};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/template");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <project-name>", args[0]);
        process::exit(1);
    }

    let project_name = &args[1];
    let safe_name = project_name.replace("-", "_");

    let project_dir = Path::new(project_name);
    if project_dir.exists() {
        eprintln!("Error: Directory '{}' already exists.", project_name);
        process::exit(1);
    }

    println!("Creating MRE Rust Framework project: {}", project_name);
    fs::create_dir(project_dir).expect("Failed to create project directory");

    TEMPLATE.extract(&project_dir).expect("Failed to extract template files");

    // Modify Cargo.toml
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path).unwrap();
    let updated_cargo = cargo_toml.replace("name = \"my_mre_app\"", &format!("name = \"{}\"", project_name));
    fs::write(cargo_toml_path, updated_cargo).unwrap();

    // Modify Makefile
    let makefile_path = project_dir.join("Makefile");
    let makefile = fs::read_to_string(&makefile_path).unwrap();
    let updated_makefile = makefile
        .replace("APP_FILE_NAME = MyMREApp", &format!("APP_FILE_NAME = {}", project_name))
        .replace(
            "RUST_LIB = target/armv5te-unknown-linux-gnueabi/release/libmy_mre_app.a",
            &format!("RUST_LIB = target/armv5te-unknown-linux-gnueabi/release/lib{}.a", safe_name),
        );
    fs::write(makefile_path, updated_makefile).unwrap();

    println!("Successfully initialized {}!", project_name);
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  make all");
}
