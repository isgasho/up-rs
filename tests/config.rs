mod common;

#[test]
fn empty_toml() {
    let fixtures_dir = common::fixtures_dir().join("blank_config");
    let mut cmd = common::up_cmd();
    cmd.args(
        [
            "-c",
            fixtures_dir.join("up.toml").to_str().unwrap(),
            "date",
        ]
        .iter(),
    );
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert_eq!(
        cmd_output.status.success(),
        true,
        "\n Update command should pass with empty toml as there are no required options.",
    );
}

#[test]
fn basic_toml() {
    let fixtures_dir = common::fixtures_dir().join("basic_config");
    let mut cmd = common::up_cmd();
    cmd.args(
        [
            "-c",
            fixtures_dir.join("up.toml").to_str().unwrap(),
            "date",
        ]
        .iter(),
    );
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    // TODO(gib): Why is this test passing with unknown keys in the up.toml?
    assert_eq!(
        cmd_output.status.success(),
        true,
        "\n Update command should pass with basic toml.",
    );
}
