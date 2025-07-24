#![windows_subsystem = "windows"]

use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};
use std::{
    fs::read_to_string, os::windows::process::CommandExt, path::PathBuf, process::Command,
    time::Duration,
};

fn main() {
    Command::new("../../Gw2-64.exe")
        .args(&[
            "-provider",
            "Portal",
            "-ignorecoherentgpucrash",
            "-autologin",
        ])
        .creation_flags(0x08000000)
        .spawn()
        .ok();

    let process = OwnedProcess::find_first_by_name("Gw2-64").unwrap();

    //Quick and easy way to wait until the user has launched the game.
    if process
        .wait_for_module_by_name(
            "D3DCOMPILER_47.dll",
            Duration::from_millis(u64::max_value()),
        )
        .is_err()
    {
        println!("Could not find D3DCOMPILER_47.");
        return;
    }
    //Just to make sure
    std::thread::sleep(Duration::from_secs(3));

    let syringe = Syringe::for_process(process);

    let dlls = get_dlls();
    let exes = get_exes();

    for dll in dlls {
        if dll.as_os_str().len() > 3 {
            inject_dll(&syringe, dll);
        }
    }
    for exe in exes {
        if exe.as_os_str().len() > 3 {
            run_exe(exe);
        }
    }
}

fn inject_dll(syringe: &Syringe, path: PathBuf) {
    match syringe.inject(path) {
        Ok(_) => println!("success"),
        Err(e) => println!("{}", &e.to_string()),
    }
}

fn run_exe(path: PathBuf) {
    Command::new(path).creation_flags(0x08000000).spawn().ok();
}

fn get_dlls() -> Vec<PathBuf> {
    if let Ok(contents) = read_to_string("dlls.txt") {
        println!("READ THE FILE: {}", contents);
        return contents
            .split(|b| b == '\n')
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}
fn get_exes() -> Vec<PathBuf> {
    if let Ok(contents) = read_to_string("exes.txt") {
        return contents
            .split(|b| b == '\n')
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}
