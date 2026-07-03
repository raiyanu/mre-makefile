use include_dir::{include_dir, Dir};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};
use std::convert::TryInto;

static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/template");
static SDK_BUNDLE: Dir = include_dir!("$CARGO_MANIFEST_DIR/sdk_bundle");

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn patch_vxp(mut data: Vec<u8>, imsi: &str) -> Vec<u8> {
    let imsi_str = format!("9{}", imsi);
    let imsi_bytes = imsi_str.as_bytes();
    
    let tag_table_offset = read_u32(&data, data.len() - 12) as usize;
    let mut pos = tag_table_offset;

    while pos < data.len() {
        let field_id = read_u32(&data, pos);
        if field_id == 0 {
            break;
        }
        pos += 4;
        
        let mut field_len = read_u32(&data, pos) as usize;
        let len_pos = pos;
        pos += 4;
        
        if field_len == 0 {
            continue;
        }
        
        if field_id == 2 {
            data[pos..pos + 4].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        } else if field_id == 0x12 {
            write_u32(&mut data, len_pos, imsi_bytes.len() as u32);
            data.splice(pos..pos + field_len, imsi_bytes.iter().copied());
            field_len = imsi_bytes.len();
        }
        
        pos += field_len;
    }
    
    data
}

fn print_usage(bin_name: &str) {
    eprintln!("cargo-mre - MRE Rust Framework Tool");
    eprintln!("Usage:");
    eprintln!("  {} new <project-name>", bin_name);
    eprintln!("  {} build", bin_name);
    process::exit(1);
}

fn new_project(project_name: &str) {
    let project_dir = Path::new(project_name);
    if project_dir.exists() {
        eprintln!("Error: Directory '{}' already exists.", project_name);
        process::exit(1);
    }

    println!("Creating pristine MRE Rust project: {}", project_name);
    fs::create_dir(project_dir).expect("Failed to create project directory");

    TEMPLATE.extract(&project_dir).expect("Failed to extract template files");

    // Rename Cargo.toml.tmpl to Cargo.toml
    fs::rename(project_dir.join("Cargo.toml.tmpl"), project_dir.join("Cargo.toml")).unwrap();

    // Modify Cargo.toml
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path).unwrap();
    let updated_cargo = cargo_toml.replace("name = \"my_mre_app\"", &format!("name = \"{}\"", project_name));
    fs::write(cargo_toml_path, updated_cargo).unwrap();

    println!("Successfully initialized {}!", project_name);
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  cargo mre build");
}

fn build_project() {
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_else(|_| {
        eprintln!("Cargo.toml not found. Are you in a project directory?");
        process::exit(1);
    });
    
    let project_name = cargo_toml.lines().find(|l| l.starts_with("name = "))
        .and_then(|l| l.split('"').nth(1))
        .expect("Could not parse project name from Cargo.toml");
    
    let safe_name = project_name.replace("-", "_");

    println!("Building MRE project: {}", project_name);

    // 1. Extract SDK bundle to target/mre-sdk
    let sdk_dir = Path::new("target/mre-sdk");
    if !sdk_dir.exists() {
        fs::create_dir_all(sdk_dir).unwrap();
        SDK_BUNDLE.extract(sdk_dir).expect("Failed to extract SDK bundle");
    }

    // 2. Run cargo build
    println!("Compiling Rust code...");
    let status = Command::new("cargo")
        .args(&["build", "--target", "armv5te-unknown-linux-gnueabi", "--release"])
        .status()
        .expect("Failed to run cargo build");
    if !status.success() {
        process::exit(1);
    }

    // 3. Compile C shims
    println!("Compiling SDK shims...");
    let shims = ["c_fix.c", "diag.c", "gccmain.c"];
    for shim in shims.iter() {
        let shim_path = sdk_dir.join(format!("sdk/shims/{}", shim));
        let obj_path = sdk_dir.join(format!("{}.o", shim));
        let status = Command::new("arm-none-eabi-gcc")
            .args(&[
                "-c", "-fpic", "-march=armv5te", "-fvisibility=hidden", "-Os", "-mlittle-endian",
                "-I", "target/mre-sdk/sdk/include",
                "-I", "target/mre-sdk/sdk/shims/include",
                "-I", "target/mre-sdk/sdk/shims/ResID",
                "-I", "target/mre-sdk/sdk/shims",
                "-D", "_MINIGUI_LIB_", "-D", "_USE_MINIGUIENTRY", "-D", "_NOUNIX_",
                "-D", "_FOR_WNC", "-D", "__MRE_SDK__", "-D", "__MRE_VENUS_NORMAL__",
                "-D", "__MMI_MAINLCD_240X320__", "-D", "MRE", "-D", "GCC", "-D", "__MRE_COMPILER_GCC__",
                "-fdata-sections", "-ffunction-sections",
                shim_path.to_str().unwrap(),
                "-o", obj_path.to_str().unwrap()
            ])
            .status()
            .expect("Failed to run arm-none-eabi-gcc compiler");
        if !status.success() {
            process::exit(1);
        }
    }

    // 4. Link
    println!("Linking binary...");
    let axf_path = sdk_dir.join(format!("{}.axf", project_name));
    let rust_lib_path = format!("target/armv5te-unknown-linux-gnueabi/release/lib{}.a", safe_name);
    
    let mut link_args = vec![
        "-o".to_string(), axf_path.to_str().unwrap().to_string(),
        "target/mre-sdk/c_fix.c.o".to_string(),
        "target/mre-sdk/diag.c.o".to_string(),
        "target/mre-sdk/gccmain.c.o".to_string(),
        rust_lib_path,
        "-lstdc++".to_string(),
    ];

    let lib_entries = fs::read_dir(sdk_dir.join("sdk/lib")).unwrap();
    for entry in lib_entries {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("a") {
            link_args.push(entry.path().to_str().unwrap().to_string());
        }
    }

    link_args.extend_from_slice(&[
        "-fno-threadsafe-statics".to_string(),
        "-Wl,--gc-sections".to_string(),
        "-fpic".to_string(),
        "-fpcc-struct-return".to_string(),
        "--disable-libstdcxx-verbose".to_string(),
        "-pie".to_string(),
        "-lm".to_string(),
        "-T".to_string(), "target/mre-sdk/sdk/scat.ld".to_string(),
    ]);

    let status = Command::new("arm-none-eabi-gcc")
        .args(&link_args)
        .status()
        .expect("Failed to run arm-none-eabi-gcc linker");
    if !status.success() {
        process::exit(1);
    }

    // 5. Objcopy
    println!("Generating VXP...");
    let vxp_path = sdk_dir.join(format!("{}.vxp", project_name));
    let status = Command::new("objcopy")
        .args(&[
            "-I", "elf32-little",
            "--add-section", ".vm_res=target/mre-sdk/sdk/resource.bin",
            axf_path.to_str().unwrap(),
            vxp_path.to_str().unwrap()
        ])
        .status()
        .expect("Failed to run objcopy");
    if !status.success() {
        process::exit(1);
    }

    // 6. append tags.bin
    let mut vxp_data = fs::read(&vxp_path).unwrap();
    let tags_data = fs::read(sdk_dir.join("sdk/tags.bin")).unwrap_or_else(|_| vec![]);
    vxp_data.extend_from_slice(&tags_data);

    // 7. vxpatch
    let manifest = fs::read_to_string("manifest.json").unwrap_or_default();
    let imsi_val = manifest
        .lines()
        .find(|line| line.contains("\"imsi\""))
        .and_then(|line| line.split('"').nth(3))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "000000000000000".to_string());
    let imsi = imsi_val.trim();
    let imsi = imsi.trim();
    let patched_data = patch_vxp(vxp_data, imsi);
    
    // Write final output to project directory
    let out_file = format!("{}.vxp", project_name);
    fs::write(&out_file, patched_data).expect("Failed to write final vxp");

    println!("Successfully built {}!", out_file);
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "mre" {
        args.remove(1);
    }
    
    if args.len() < 2 {
        print_usage(&args[0]);
    }

    let command = &args[1];
    match command.as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("Usage: {} new <project-name>", args[0]);
                process::exit(1);
            }
            new_project(&args[2]);
        }
        "build" => {
            build_project();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
        }
    }
}
