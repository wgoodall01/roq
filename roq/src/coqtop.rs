use eyre::{bail, eyre, Result, WrapErr};

use std::ffi::OsString;
use std::path::Path;

/// A Coqtop binary.
pub struct Coqtop {
    binary_path: OsString,
}

impl Coqtop {
    /// Create a [`Coqtop`] which uses the default `coqtop` binary installed on this
    /// system.
    pub fn new() -> Coqtop {
        Coqtop {
            binary_path: OsString::from("coqtop"),
        }
    }

    /// Create a [`Coqtop`] which uses a specific binary.
    pub fn with_binary<P: Into<OsString>>(binary_path: P) -> Coqtop {
        Coqtop {
            binary_path: binary_path.into(),
        }
    }

    fn cmd(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.binary_path);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .arg("-q") // Don't load the rcfile
            .env_clear()
            .env("HOME", "/roq-fake-home");
        cmd
    }

    /// Run a batch of Coq vernacular files, returning any successful output.
    pub fn run_batch(&self, batch: &[&Path]) -> Result<String> {
        let mut cmd = self.cmd();

        // Append each file from the batch as args.
        for path in batch {
            // Check that the file has a '.v' extension.
            if match path.extension() {
                None => true,
                Some(p) if p != "v" => true,
                _ => false,
            } {
                bail!("File must have '.v' extension: {:?}", path)
            }

            cmd.arg("-l");
            cmd.arg(path);
        }

        // Run the command.
        let child = cmd.spawn().wrap_err("Failed to spawn Coqtop")?;
        let output = child
            .wait_with_output()
            .wrap_err("Failed to wait for Coqtop to exit")?;

        // Check for failure.
        if !output.status.success() {
            // Convert stderr to a string (lossy), wrap in an error.
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Return failure.
            let err = eyre!("{}", stderr).wrap_err("Coqtop failed");
            return Err(err);
        }

        // Convert stdout to a string and return.
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

impl Default for Coqtop {
    fn default() -> Self {
        Self::new()
    }
}
