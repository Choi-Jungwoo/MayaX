use dll_syringe::{
    Syringe,
    process::{OwnedProcess, Process},
};
use std::{env, path::Path, process::Command, time::Duration};

fn main() {
    let maya_home = env::var("MAYA_HOME").expect("Environment variable MAYA_HOME is not set");
    let maya_path = Path::new(&maya_home).join("bin/maya.exe");
    let maya_process = Command::new(maya_path)
        .spawn()
        .expect("Failed to start Maya process");

    let maya_pid = maya_process.id();
    let target_process = OwnedProcess::from_pid(maya_pid).expect("Failed to find Maya process");

    target_process
        .wait_for_module_by_name("OpenMaya.dll", Duration::from_secs(10))
        .expect("Failed to find OpenMaya.dll");

    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let dll_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");
    let dll_path = dll_dir.join("mayax_runtime.dll");

    let syringe = Syringe::for_process(target_process);
    let _injected_payload = syringe.inject(dll_path).unwrap();
}
