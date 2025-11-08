use assert_cmd::Command;
use tempfile::TempDir;

pub struct TestEnv {
    temp_config: TempDir,
}

impl TestEnv {
    pub fn new() -> Self {
        let temp_config = TempDir::new().expect("temp dir");
        Self { temp_config }
    }

    pub fn tt_cmd(&self) -> Command {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tt"));
        cmd.env("TT_CONFIG_DIR", self.temp_config.path());
        // Also ensure clap won't read user config via XDG vars.
        cmd.env_remove("XDG_CONFIG_HOME");
        cmd.env_remove("APPDATA");
        cmd
    }
}
