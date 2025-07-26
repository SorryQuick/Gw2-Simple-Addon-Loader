#![windows_subsystem = "windows"]

use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};
use std::{
    fs::{read_to_string, File},
    io::{BufRead, BufReader, Read, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() {
    let mut f = File::create("log.txt").unwrap();
    log(&mut f, "Launching gw2...");

    let mut gw2_child = Command::new("../../Gw2-64.exe")
        .args(&[
            "-provider",
            "Portal",
            "-ignorecoherentgpucrash",
            "-autologin",
        ])
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();

    log(&mut f, "Finding game process...");
    let process = OwnedProcess::find_first_by_name("Gw2-64").unwrap();

    log(&mut f, "Waiting for user to launch the game...");
    //Quick and easy way to wait until the user has launched the game.
    process
        .wait_for_module_by_name(
            "D3DCOMPILER_47.dll",
            Duration::from_millis(u64::max_value()),
        )
        .ok();

    //Just to make sure
    std::thread::sleep(Duration::from_secs(3));

    //std::thread::sleep(Duration::from_secs(30));

    log(&mut f, "Game has been launched.");

    let syringe = Syringe::for_process(process);

    let dlls = get_dlls(&mut f);
    let exes = get_exes(&mut f);

    for dll in dlls {
        if dll.as_os_str().len() > 3 {
            inject_dll(&syringe, dll, &mut f);
        }
    }

    let mut children = Vec::with_capacity(exes.len());

    for exe in exes {
        if exe.as_os_str().len() > 3 {
            let mut f_cloned = f.try_clone().unwrap();
            children.push(run_exe(exe, &mut f_cloned.try_clone().unwrap()));
        }
    }

    gw2_child.wait();

    //Game is closed, gotta kill every exe launched to close the wine prefix/instance cleanly
    //For example, Blish like to stick around if not killed.
    for child in children {
        if let Some(mut child) = child {
            child.kill().ok();
        }
    }
}

fn inject_dll(syringe: &Syringe, path: PathBuf, f: &mut File) {
    match syringe.inject(&path) {
        Ok(_) => log(
            f,
            &format!("Successfully injected {}", path.to_str().unwrap()),
        ),
        Err(e) => log(
            f,
            &format!(
                "Failed injecting {}. Error: {}",
                path.to_str().unwrap(),
                e.to_string()
            ),
        ),
    }
}

fn run_exe(path: PathBuf, f: &mut File) -> Option<Child> {
    match Command::new(&path)
        .creation_flags(0x08000000)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            log(
                f,
                &format!("Successfully launched {}", path.to_str().unwrap()),
            );
            Some(child)
        }
        Err(e) => {
            log(
                f,
                &format!("Failed to launch {}: {}", path.to_str().unwrap(), e),
            );
            None
        }
    }
}

fn get_dlls(f: &mut File) -> Vec<PathBuf> {
    if let Ok(contents) = read_to_string("dlls.txt") {
        log(f, "Read dlls.");
        return contents
            .split(|b| b == '\n')
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}
fn get_exes(f: &mut File) -> Vec<PathBuf> {
    if let Ok(contents) = read_to_string("exes.txt") {
        log(f, "Read exes.");
        return contents
            .split(|b| b == '\n')
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}

fn log(f: &mut File, s: &str) {
    println!("{}", s);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    f.write_all(
        format!(
            "[{}] [Gw2SimpleAddonLoader] {}\n",
            format!("{}", now.as_secs()),
            s
        )
        .as_bytes(),
    );
}
