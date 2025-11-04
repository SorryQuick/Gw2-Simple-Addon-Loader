#![windows_subsystem = "windows"]

use crate::logging::log;
use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};
use logging::clean_logs;
use std::{
    env::{self, current_exe},
    fs::read_to_string,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::OnceLock,
    time::Duration,
};

mod logging;

static EXE_DIR: OnceLock<PathBuf> = OnceLock::new();

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
            })
            .unwrap_or("Unknown panic");

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        log(&format!("PANIC at {}: {}", location, payload));
    }));

    let mut args = env::args().skip(1).collect::<Vec<String>>();

    let exe_path = EXE_DIR.get_or_init(|| {
        current_exe()
            .expect("Failed to get current exe path")
            .parent()
            .expect("Failed to get exe directory")
            .to_path_buf()
    });

    clean_logs();

    log(&format!(
        "Running from addon directory: \"{}\"",
        exe_path.to_str().unwrap()
    ));

    // Guild Wars 2 binary should be 2 directories up
    let mut gw2_path = exe_path.clone();
    gw2_path.pop();
    gw2_path.pop();
    gw2_path.push("Gw2-64.exe");
    log(&format!(
        "Looking for Gw2-64.exe on path: \"{}\"",
        gw2_path.to_str().unwrap()
    ));

    log("Launching Guild Wars 2...");

    //Default arguments if they aren't present
    for required in ["-ignorecoherentgpucrash", "-autologin"] {
        if !args.iter().any(|arg| arg == required) {
            args.push(required.into());
        }
    }
    let mut gw2_child = if env::var("USE_STEAM_LOGIN").unwrap_or("0".into()) == "1" {
        log("Game is running with Steam.");
        let app_id = "1284210";
        if !args.iter().any(|arg| arg == "-provider") {
            args.push("-provider".into());
            args.push("Steam".into());
        }
        Command::new(gw2_path.to_str().unwrap())
            .env("SteamAppId", app_id)
            .env("SteamGameId", app_id)
            .args(&args)
            .spawn()
            .unwrap()
    } else {
        log("Game is NOT running with Steam.");
        if !args.iter().any(|arg| arg == "-provider") {
            args.push("-provider".into());
            args.push("Portal".into());
        }
        Command::new(gw2_path.to_str().unwrap())
            .args(&args)
            .spawn()
            .unwrap()
    };

    let dlls = get_dlls(&exe_path);
    let exes = get_exes(&exe_path);

    log(&format!(
        "Found {} dlls and {} exes",
        dlls.len(),
        exes.len()
    ));

    log("Finding game process...");
    let process = OwnedProcess::find_first_by_name("Gw2-64").unwrap();

    log("Waiting for game to be fully running...");
    //Quick and easy way to wait until the user has launched the game.
    process
        .wait_for_module_by_name("d3d11.dll", Duration::from_millis(u64::max_value()))
        .ok();

    //Just to make sure
    std::thread::sleep(Duration::from_secs(3));

    log("Game has been launched.");

    let syringe = Syringe::for_process(process);

    for dll in dlls {
        if dll.as_os_str().len() > 3 {
            inject_dll(&syringe, dll);
        }
    }

    let mut children = Vec::with_capacity(exes.len());

    for exe in exes {
        if exe.as_os_str().len() > 3 {
            children.push(run_exe(exe));
        }
    }

    gw2_child.wait().ok();

    //Game is closed, gotta kill every exe launched to close the wine prefix/instance cleanly
    //For example, Blish like to stick around if not killed.
    for child in children {
        if let Some(mut child) = child {
            child.kill().ok();
        }
    }
}

fn inject_dll(syringe: &Syringe, path: PathBuf) {
    match syringe.inject(&path) {
        Ok(_) => log(&format!("Successfully injected {}", path.to_str().unwrap())),
        Err(e) => log(&format!(
            "Failed injecting {}. Error: {}",
            path.to_str().unwrap(),
            e.to_string()
        )),
    }
}

fn run_exe(path: PathBuf) -> Option<Child> {
    match Command::new(&path)
        .creation_flags(0x08000000)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            log(&format!("Launched {}", path.to_str().unwrap()));
            Some(child)
        }
        Err(e) => {
            log(&format!(
                "Failed to launch {}: {}",
                path.to_str().unwrap(),
                e
            ));
            None
        }
    }
}

fn get_dlls(binary_path: &PathBuf) -> Vec<PathBuf> {
    let mut dlls_path = binary_path.clone();
    dlls_path.push("dlls.txt");
    log(&format!(
        "Trying to read dlls from {}",
        dlls_path.to_str().unwrap()
    ));
    if let Ok(contents) = read_to_string(dlls_path) {
        return contents
            .split(|b| b == '\n')
            .filter(|s| !s.trim().is_empty())
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}

fn get_exes(binary_path: &PathBuf) -> Vec<PathBuf> {
    let mut exes_path = binary_path.clone();
    exes_path.push("exes.txt");
    log(&format!(
        "Trying to read exes from {}",
        exes_path.to_str().unwrap()
    ));
    if let Ok(contents) = read_to_string(exes_path) {
        return contents
            .split(|b| b == '\n')
            .filter(|s| !s.trim().is_empty())
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}
