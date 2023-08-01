use bindgen::callbacks::{ItemInfo, ParseCallbacks};
use std::env;
use std::io::Write;
use std::path::PathBuf;

// Cargo provides the crate version from Cargo.toml in the environment.
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn version_prefix() -> String {
    format!("verslib_{}_", VERSION.replace(".", "_"))
}

fn main() {
    // This is the standard use of bindgen, except there is no "wrapper.h",
    // and we include the RenameCallbacks.
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("c/include/verslib.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(RenameCallbacks {}))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // This is a standard way of building a simple native library.
    cc::Build::new()
        .file("c/src/verslib.c")
        .compile("verslib");

    // Rename the generated library.
    std::fs::rename(
        out_path.join("libverslib.a"),
        out_path.join("libverslib_orig.a"),
    )
    .unwrap();

    // Run nm on the generated library.
    let output = std::process::Command::new("nm")
        .args([out_path.join("libverslib_orig.a")])
        .output()
        .expect("failed to run nm");
    if !output.status.success() {
        panic!("nm failed");
    }

    // FIXME: Here we should parse the output from nm to generate a
    // list of symbols to be renamed. Typically that would be all the
    // global symbols defined in the library, and perhaps also a few
    // undefined symbols that the library expects the user to provide,
    // so this will probably have to be adjusted depending on the
    // library. I am unsure what to do about common symbols and
    // versioned symbols and other things that might appear in the
    // output from nm. For this demo, in which in the C code defines a
    // single function, we can just do this:
    let syms = ["verslib_version".to_string()];

    // Generate a file for objcopy containing "old new" in each line.
    let prefix = version_prefix();
    let symfile = out_path.join("verslib_syms");
    {
        let mut file = std::fs::File::create(&symfile).unwrap();
        for sym in syms.iter() {
            file.write_all(format!("{} {}{}\n", sym, prefix, sym).as_bytes())
                .unwrap();
        }
    }

    // Run objcopy to create a new version of the library with renamed symbols.
    let status = std::process::Command::new("objcopy")
        .args([
            "--redefine-syms",
            symfile.to_str().unwrap(),
            out_path.join("libverslib_orig.a").to_str().unwrap(),
            out_path.join("libverslib.a").to_str().unwrap(),
        ])
        .status()
        .expect("failed to execute process");
    if !status.success() {
        panic!("objcopy failed");
    }
}

#[derive(Debug)]
struct RenameCallbacks {}

impl ParseCallbacks for RenameCallbacks {
    fn generated_link_name_override(&self, info: ItemInfo<'_>) -> Option<String> {
        Some(version_prefix() + info.name)
    }
}
