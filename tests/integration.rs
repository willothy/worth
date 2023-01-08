use std::{path::PathBuf, process::Command};

use serial_test::serial;

fn runner(category: &str, name: &str) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let file = dir.join(&category).join(&name).with_extension("porth");
    let args_file = dir.join(&category).join(&name).with_extension("txt");
    let args = std::fs::read_to_string(&args_file).ok();
    let out_file = dir
        .join("".to_string() + category + "/" + name)
        .with_extension("");

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
    let mut output = Command::new(&out_file);
    if let Some(args) = &args {
        //output.arg("--");
        output.args(args.lines().map(|s| s.trim()).collect::<Vec<&str>>());
    }
    let output = output.output().expect("failed to execute process");
    assert_eq!(
        output.status.success(),
        true,
        "\n\n------ Test Error ------\nProgram {} exited with non-zero status:\n\n{}\n---- End Test Error ----\n------- Test Out -------\nProgram {} exited with non-zero status:\n\n{}\n----- End Test Out -----\n",
        &name,
        unsafe { String::from_utf8_unchecked(output.stderr) },
        &name,
        unsafe { String::from_utf8_unchecked(output.stdout) }
    );
    let mut sim = test_bin::get_test_bin("worthc");
    sim.arg(file).arg("S");
    if let Some(args) = &args {
        sim.arg("--");
        sim.args(args.lines().map(|s| s.trim()).collect::<Vec<&str>>());
    }
    let sim_output = sim.output().expect("failed to execute process");
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
fn hello_world() {
    runner("programs", "hello");
}

#[test]
fn memory() {
    runner("programs", "memory");
}

#[test]
fn bitwise() {
    runner("programs", "bitwise");
}

#[test]
fn rule110() {
    runner("programs", "rule110");
}

#[test]
fn string() {
    runner("programs", "string");
}

#[test]
fn char() {
    runner("programs", "char");
}

#[test]
fn include() {
    runner("programs", "include");
}

#[test]
fn math() {
    runner("programs", "math");
}

#[test]
fn args() {
    runner("programs", "args");
}

#[test]
fn euler1() {
    runner("euler", "problem01");
}

#[test]
fn euler2() {
    runner("euler", "problem02");
}
