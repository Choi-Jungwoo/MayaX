use std::{env, path::Path};

fn get_maya_home_path() -> String {
    String::from(Path::new(env!("MAYA_HOME")).to_str().unwrap())
}

fn main() {
    let maya_home_path = get_maya_home_path();

    println!("cargo:rustc-link-search=native={}/lib", maya_home_path);
    println!("cargo:rustc-link-lib=Foundation");
    println!("cargo:rustc-link-lib=OpenMaya");
}
