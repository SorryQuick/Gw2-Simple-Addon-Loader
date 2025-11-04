use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use std::{
    fs::{self, create_dir_all, remove_file, File, OpenOptions},
    io::Write,
    sync::OnceLock,
};

use crate::EXE_DIR;
static FILE: OnceLock<File> = OnceLock::new();

pub fn log(message: &str) {
    let mut file = FILE.get_or_init(|| {
        let logs_dir = EXE_DIR.get().unwrap().join("logs");
        create_dir_all(&logs_dir).expect("Failed to create logs directory");

        let filename = format!("loader-{}.log", Local::now().format("%Y-%m-%d_%H-%M-%S"));
        let filepath = logs_dir.join(filename);

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)
            .expect("Failed to open log file")
    });

    let now = Local::now();
    let line = format!(
        "[{}] [Gw2SimpleAddonLoader] {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        message
    );
    print!("{}", line);

    file.write_all(line.as_bytes()).ok();
}

/// Deletes log files older than 24 hours based on filename timestamps.
/// Assumes log files named loader-YYYY-MM-DD_HH-MM-SS.log.
pub fn clean_logs() {
    let logs_dir = EXE_DIR.get().unwrap().join("logs");
    let cutoff = Local::now() - Duration::hours(24);
    if let Ok(entries) = fs::read_dir(logs_dir) {
        for entry in entries.flatten() {
            if let Some(fname) = entry.file_name().to_str() {
                if let Some(datetime_str) = fname
                    .rsplit_once('-')
                    .map(|(_, tail)| tail.strip_suffix(".log"))
                    .flatten()
                {
                    if let Ok(naive_dt) =
                        NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d_%H-%M-%S")
                    {
                        let file_dt = Local
                            .from_local_datetime(&naive_dt)
                            .single()
                            .expect("Ambiguous or invalid local time");
                        if file_dt < cutoff {
                            remove_file(entry.path()).ok();
                        }
                    }
                }
            }
        }
    }
}
