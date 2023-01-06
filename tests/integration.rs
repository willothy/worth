use std::{path::PathBuf, process::Command};

use serial_test::serial;

fn runner(name: String) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let file = dir.join("programs").join(&name).with_extension("worth");
    let out_file = dir.join("tmp_test").with_extension("");
    println!("file: {:?}", file);
    let output = test_bin::get_test_bin("worthc")
        .arg(&file)
        .args(["C", "-o"])
        .arg(&out_file)
        .output()
        .expect("failed to execute process");
    assert_eq!(
        output.status.success(),
        true,
        "Compiler exited with non-zero status for program"
    );
    let output = Command::new(&out_file)
        .output()
        .expect("failed to execute process");
    assert_eq!(
        output.status.success(),
        true,
        "Program {} exited with non-zero status",
        &name
    );
    let sim_output = test_bin::get_test_bin("worthc")
        .arg(file)
        .arg("S")
        .output()
        .expect("failed to execute process");
    assert_eq!(
        sim_output.status.success(),
        true,
        "Sim for {} exited with non-zero status",
        &name
    );

    // TODO: Fix the sim
    // assert!(sim_output.stdout == output.stdout);
    // assert!(sim_output.stderr == output.stderr);

    // Remove the tmp_test and tmp_test.asm files
    std::fs::remove_file(&out_file).expect("Could not remove tmp_test file");
    std::fs::remove_file(out_file.with_extension("asm"))
        .expect("Could not remove tmp_test.asm file");
}

#[test]
#[serial]
fn hello_world() {
    runner("hello".to_string());
}

#[test]
#[serial]
fn memory() {
    runner("memory".to_string());
}
