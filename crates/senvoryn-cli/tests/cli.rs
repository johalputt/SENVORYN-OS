//! Exercises the command-line tool end to end.
//!
//! The tool is a development aid, so the thing worth testing is that it
//! produces the decision it says it produces, and that it says plainly what
//! it is not.

use std::process::Command;

const EXAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/cli");

fn run(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_senvoryn-scp"))
        .args(arguments)
        .output()
        .expect("the tool runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn a_permitted_request_produces_a_permitted_record() {
    let (ok, stdout, stderr) = run(&[
        "--request",
        &format!("{EXAMPLES}/request.json"),
        "--context",
        &format!("{EXAMPLES}/context.json"),
        "--capability",
        &format!("{EXAMPLES}/capability.json"),
    ]);
    assert!(ok, "the tool failed: {stderr}");
    assert!(stdout.contains("\"status\":\"PERMIT\""), "unexpected output: {stdout}");
    assert!(stdout.contains("\"reason_codes\":[\"ALL_POLICY_GATES_PASSED\"]"));
    assert!(stdout.contains("\"effect_performed\":false"));
    assert!(stdout.contains("\"capability_scope_ref\":\"cap:example:0001\""));
}

#[test]
fn a_request_without_a_capability_is_denied() {
    let (ok, stdout, stderr) = run(&[
        "--request",
        &format!("{EXAMPLES}/request.json"),
        "--context",
        &format!("{EXAMPLES}/context.json"),
    ]);
    assert!(ok, "the tool failed: {stderr}");
    assert!(stdout.contains("\"status\":\"DENY\""), "unexpected output: {stdout}");
    assert!(stdout.contains("\"reason_codes\":[\"MISSING_CAPABILITY\"]"));
    assert!(stdout.contains("\"effect_performed\":false"));
}

#[test]
fn the_output_is_canonical_json() {
    let (ok, stdout, stderr) = run(&[
        "--request",
        &format!("{EXAMPLES}/request.json"),
        "--context",
        &format!("{EXAMPLES}/context.json"),
        "--capability",
        &format!("{EXAMPLES}/capability.json"),
    ]);
    assert!(ok, "the tool failed: {stderr}");
    let document = senvoryn_canonical::parse(&stdout).expect("the output parses");
    assert_eq!(
        senvoryn_canonical::encode_to_string(&document),
        stdout.trim_end(),
        "the output is not in canonical form"
    );
}

#[test]
fn the_help_says_it_is_not_an_scp() {
    let (ok, stdout, stderr) = run(&["--help"]);
    assert!(ok, "the tool failed: {stderr}");
    // The help is wrapped for reading, so a phrase can straddle a line break.
    // Matching against collapsed whitespace keeps the test about what the
    // help says rather than about how it happens to be laid out.
    let collapsed = stdout.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        collapsed.contains("NOT AN SCP"),
        "the help does not say what the tool is not: {stdout}"
    );
    assert!(
        collapsed.contains("must not be deployed as a System Control Plane"),
        "the help does not say what the tool is not: {stdout}"
    );
}

#[test]
fn an_unknown_argument_is_refused() {
    let (ok, _stdout, stderr) = run(&["--unknown", "value"]);
    assert!(!ok, "the tool accepted an unknown argument");
    assert!(stderr.contains("unknown argument"));
}

#[test]
fn a_missing_file_is_refused() {
    let (ok, _stdout, stderr) = run(&[
        "--request",
        &format!("{EXAMPLES}/no-such-file.json"),
        "--context",
        &format!("{EXAMPLES}/context.json"),
    ]);
    assert!(!ok, "the tool accepted a missing file");
    assert!(stderr.contains("could not read"));
}
