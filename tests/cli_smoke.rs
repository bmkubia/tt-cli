mod common;

use common::TestEnv;
use predicates::prelude::*;

#[test]
fn running_without_question_requires_input() {
    let env = TestEnv::new();
    env.tt_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Please provide a question"));
}

#[test]
fn config_command_reports_missing_setup() {
    let env = TestEnv::new();
    env.tt_cmd()
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("No configuration found"));
}
