use std::{path::PathBuf, process::Command};

use serial_test::serial;

fn runner(name: String) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let file = dir.join("programs").join(&name).with_extension("porth");
    let out_file = dir.join("tmp_test").with_extension("");
    println!("file: {:?}", file);
    let output = test_bin::get_test_bin("worthc")
        .arg(&file)
        .args(["build", "-o"])
        .arg(&out_file)
        .output()
        .expect("failed to execute process");
    assert_eq!(
        output.status.success(),
        true,
        "\n\n---- Compiler Error ----\nCompiler exited with non-zero status for program:\n\n{}\n-- End Compiler Error --\n\n",
        unsafe { String::from_utf8_unchecked(output.stderr) }
    );
    let output = Command::new(&out_file)
        .output()
        .expect("failed to execute process");
    assert_eq!(
        output.status.success(),
        true,
        "\n\n------ Test Error ------\nProgram {} exited with non-zero status:\n\n{}\n---- End Test Error ----\n------- Test Out -------\nProgram {} exited with non-zero status:\n\n{}\n----- End Test Out -----\n",
        &name,
        unsafe { String::from_utf8_unchecked(output.stderr) },
        &name,
        unsafe { String::from_utf8_unchecked(output.stdout) }
    );
    let sim_output = test_bin::get_test_bin("worthc")
        .arg(file)
        .arg("S")
        .output()
        .expect("failed to execute process");
    assert_eq!(
        sim_output.status.success(),
        true,
        "\n\n------- Sim Error ------\nSim for {} exited with non-zero status:\n\n{}\n----- End Sim Error ----\n",
        &name,
        unsafe { String::from_utf8_unchecked(sim_output.stderr) }
    );

    assert!(
        sim_output.stdout == output.stdout,
        "\nSim:\nStdout:\n{}\n\nStderr:\n{}\n\nTest:\nStdout:\n{}\n\nStderr:\n{}\n",
        unsafe { String::from_utf8_unchecked(sim_output.stdout) },
        unsafe { String::from_utf8_unchecked(sim_output.stderr) },
        unsafe { String::from_utf8_unchecked(output.stdout) },
        unsafe { String::from_utf8_unchecked(output.stderr) }
    );
    assert!(sim_output.stderr == output.stderr);

    // Remove the tmp_test file
    std::fs::remove_file(&out_file).expect("Could not remove tmp_test file");
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

#[test]
#[serial]
fn bitwise() {
    runner("bitwise".to_string());
}

#[test]
#[serial]
fn rule110() {
    runner("rule110".to_string());
}

#[test]
#[serial]
fn string() {
    runner("string".to_string());
}

#[test]
#[serial]
fn char() {
    runner("char".to_string());
}

#[test]
#[serial]
fn include() {
    runner("include".to_string());
}

#[test]
#[serial]
fn math() {
    runner("math".to_string());
}
