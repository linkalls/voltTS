use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static BUILD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn sample_main_runs_with_std_runtime() {
    let _guard = BUILD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock poisoned");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest_dir.join("dist");
    if dist.exists() {
        let _ = fs::remove_dir_all(&dist);
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_voltts"));
    cmd.current_dir(&manifest_dir).arg("run");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "[info] booting VoltTS prototype (async demo)",
        ))
        .stdout(predicate::str::contains("unix epoch (ms):"))
        .stdout(predicate::str::contains("[warn] demo sleep (await)"))
        .stdout(predicate::str::contains("[info] helper start"))
        .stdout(predicate::str::contains("[warn] helper end"))
        .stdout(predicate::str::contains("hello from VoltTS async"))
        .stdout(predicate::str::contains("[info] demo done"))
        .stdout(predicate::str::contains("[error] demo complete"));
}

#[test]
fn stdlib_showcase_runs_with_relative_imports() {
    let _guard = BUILD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock poisoned");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest_dir.join("dist");
    if dist.exists() {
        let _ = fs::remove_dir_all(&dist);
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_voltts"));
    cmd.current_dir(&manifest_dir)
        .arg("run")
        .arg("tests/stdlib_showcase.vts");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stdlib showcase start"))
        .stdout(predicate::str::contains("helper tick"))
        .stdout(predicate::str::contains("helper finished"))
        .stdout(predicate::str::contains("showcase payload"))
        .stdout(predicate::str::contains("stdlib showcase done"));
}

#[test]
fn fs_runtime_writes_and_reads_files() {
    let _guard = BUILD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock poisoned");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest_dir.join("dist");
    if dist.exists() {
        let _ = fs::remove_dir_all(&dist);
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_voltts"));
    cmd.current_dir(&manifest_dir)
        .arg("run")
        .arg("tests/fs_sample.vts");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fs demo start"))
        .stdout(predicate::str::contains("hello fs runtime"))
        .stdout(predicate::str::contains("fs demo end"));

    let tmp_path = manifest_dir.join("tmp_fs.txt");
    if tmp_path.exists() {
        let _ = fs::remove_file(tmp_path);
    }
}
