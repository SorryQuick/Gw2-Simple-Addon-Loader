#![windows_subsystem = "windows"]

use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};
use std::{
    env,
    fs::{read_to_string, File},
    io::Write,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() {
    // Path to Gw2SimpleAddonLoader binary directory
    let binary_path = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    // Path to the log file
    let mut log_path = binary_path.clone();
    log_path.push("log.txt");
    // Create the log file
    let mut f = File::create(log_path.to_str().unwrap()).unwrap();
    log(
        &mut f,
        &format!(
            "Running from addon directory: \"{}\"",
            binary_path.to_str().unwrap()
        ),
    );

    // Guild Wars 2 binary should be 2 directories up
    let mut gw2_path = binary_path.clone();
    gw2_path.pop();
    gw2_path.pop();
    gw2_path.push("Gw2-64.exe");
    log(
        &mut f,
        &format!(
            "Looking for Gw2-64.exe on path: \"{}\"",
            gw2_path.to_str().unwrap()
        ),
    );

    log(&mut f, "Launching Guild Wars 2...");
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

    let dlls = get_dlls(&mut f, &binary_path);
    let exes = get_exes(&mut f, &binary_path);

    log(
        &mut f,
        &format!("Found {} dlls and {} exes", dlls.len(), exes.len()),
    );

    log(&mut f, "Finding game process...");
    let process = OwnedProcess::find_first_by_name("Gw2-64").unwrap();

    log(&mut f, "Waiting for game to be fully running...");
    //Quick and easy way to wait until the user has launched the game.
    process
        .wait_for_module_by_name("d3d11.dll", Duration::from_millis(u64::max_value()))
        .ok();

    //Just to make sure
    std::thread::sleep(Duration::from_secs(3));

    log(&mut f, "Game has been launched.");

    let syringe = Syringe::for_process(process);

    for dll in dlls {
        if dll.as_os_str().len() > 3 {
            inject_dll(&syringe, dll, &mut f);
        }
    }

    let mut children = Vec::with_capacity(exes.len());

    for exe in exes {
        if exe.as_os_str().len() > 3 {
            let f_cloned = f.try_clone().unwrap();
            children.push(run_exe(exe, &mut f_cloned.try_clone().unwrap()));
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
        Ok(child) => {
            log(f, &format!("Launched {}", path.to_str().unwrap()));
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

fn get_dlls(f: &mut File, binary_path: &PathBuf) -> Vec<PathBuf> {
    let mut dlls_path = binary_path.clone();
    dlls_path.push("dlls.txt");
    log(
        f,
        &format!("Trying to read dlls from {}", dlls_path.to_str().unwrap()),
    );
    if let Ok(contents) = read_to_string(dlls_path) {
        return contents
            .split(|b| b == '\n')
            .map(|line| PathBuf::from(line))
            .collect::<Vec<PathBuf>>();
    }
    Vec::new()
}

fn get_exes(f: &mut File, binary_path: &PathBuf) -> Vec<PathBuf> {
    let mut exes_path = binary_path.clone();
    exes_path.push("exes.txt");
    log(
        f,
        &format!("Trying to read exes from {}", exes_path.to_str().unwrap()),
    );
    if let Ok(contents) = read_to_string(exes_path) {
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
    )
    .ok();
}
