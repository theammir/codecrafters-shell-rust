//! Per-test filesystem and environment isolation.
//!
//! Every test gets its own cwd, `HOME`, and `PATH`. Nothing here touches the
//! real environment, so tests are safe to run in parallel and in any order.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// An isolated world for one shell session.
pub struct Sandbox {
    root: TempDir,
    env: BTreeMap<String, String>,
}

impl Sandbox {
    /// Create a sandbox with an empty cwd, an empty `HOME`, and a `PATH`
    /// containing a single empty directory.
    ///
    /// `PATH` deliberately excludes the real system directories: a test that
    /// needs `ls` should install a fake one, so its expectations cannot drift
    /// with whatever happens to be installed on the machine.
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("shell-test-")
            .tempdir()
            .expect("failed to create sandbox");

        for dir in ["cwd", "home", "bin"] {
            std::fs::create_dir_all(root.path().join(dir)).expect("failed to create sandbox dir");
        }

        let mut env = BTreeMap::new();
        env.insert("HOME".to_string(), path_str(&root.path().join("home")));
        env.insert("PATH".to_string(), path_str(&root.path().join("bin")));
        // Keep the shell's own line editing predictable regardless of the
        // developer's terminal.
        env.insert("TERM".to_string(), "dumb".to_string());

        Self { root, env }
    }

    /// The directory the shell starts in.
    pub fn cwd(&self) -> PathBuf {
        self.root.path().join("cwd")
    }

    /// The sandbox `HOME`.
    pub fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    /// The single directory on the sandbox `PATH`.
    pub fn bin(&self) -> PathBuf {
        self.root.path().join("bin")
    }

    /// The sandbox root. Useful for asserting on absolute paths.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// The environment the shell is spawned with.
    pub fn env(&self) -> impl Iterator<Item = (&str, &str)> {
        self.env.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Set an environment variable for the shell. Must be called before spawn.
    pub fn set_env(&mut self, key: &str, value: &str) -> &mut Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Append a directory to the sandbox `PATH`, after the existing entries.
    pub fn push_path_dir(&mut self, dir: &Path) -> &mut Self {
        let current = self.env.get("PATH").cloned().unwrap_or_default();
        let joined = if current.is_empty() {
            path_str(dir)
        } else {
            format!("{current}:{}", path_str(dir))
        };
        self.env.insert("PATH".to_string(), joined);
        self
    }

    /// Create a directory under the sandbox root, parents included.
    pub fn mkdir(&self, relative: impl AsRef<Path>) -> PathBuf {
        let path = self.root.path().join(relative);
        std::fs::create_dir_all(&path).expect("failed to create directory");
        path
    }

    /// Write a file under the sandbox root, creating parent directories.
    pub fn write_file(&self, relative: impl AsRef<Path>, contents: &str) -> PathBuf {
        let path = self.root.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create parent directory");
        }
        std::fs::write(&path, contents).expect("failed to write file");
        path
    }

    /// Read a file under the sandbox root.
    ///
    /// # Panics
    /// If the file does not exist.
    pub fn read_file(&self, relative: impl AsRef<Path>) -> String {
        let path = self.root.path().join(relative);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
    }

    /// Install an executable shell script named `name` into a `PATH` directory.
    ///
    /// Scripts are generated at runtime rather than committed, so the repository
    /// stays free of fixture binaries. `body` is the script after the shebang.
    pub fn install_executable(&self, name: &str, body: &str) -> PathBuf {
        self.install_executable_in(&self.bin(), name, body)
    }

    /// As [`Sandbox::install_executable`], but into an explicit directory. For
    /// tests that care about `PATH` precedence across several directories.
    #[expect(
        clippy::unused_self,
        reason = "method for symmetry with install_executable; dir must be a sandbox path"
    )]
    pub fn install_executable_in(&self, dir: &Path, name: &str, body: &str) -> PathBuf {
        std::fs::create_dir_all(dir).expect("failed to create bin directory");
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("failed to write executable");
        set_mode(&path, 0o755);
        path
    }

    /// Install a *non*-executable file into a `PATH` directory, to exercise the
    /// rule that such files are skipped during lookup.
    pub fn install_non_executable(&self, name: &str) -> PathBuf {
        let path = self.bin().join(name);
        std::fs::write(&path, "#!/bin/sh\necho should never run\n").expect("failed to write file");
        set_mode(&path, 0o644);
        path
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a path for use in an environment variable.
///
/// Sandbox paths are generated by `tempfile` and are always valid UTF-8, so a
/// non-UTF-8 path here means the harness is confused about its own root.
fn path_str(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("sandbox path is not utf-8: {}", path.display()))
        .to_string()
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .expect("failed to set permissions");
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}
