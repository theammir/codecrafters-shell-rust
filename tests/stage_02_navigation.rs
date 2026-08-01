//! Stage 2: `pwd` and `cd` (absolute, relative, and `~` paths).

mod harness;

use harness::{Sandbox, Session};

/// 02-navigation-01-ei0 — the `pwd` builtin.
mod navigation_01_ei0 {
    use super::*;

    // --- spec ---

    #[test]
    fn pwd_prints_the_working_directory() {
        // The shell starts in the directory it was launched from, so `pwd` must
        // report the sandbox cwd.
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("pwd", &canonical(&sandbox.cwd()));
    }

    // --- additional ---

    #[test]
    fn pwd_prints_an_absolute_path() {
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.send_line("pwd");
        shell.read_until("\n");
        let output = shell.read_until_prompt();
        assert!(
            output.starts_with('/'),
            "expected an absolute path, got {output:?}"
        );
    }

    #[test]
    fn pwd_is_stable_across_invocations() {
        // Nothing changed the directory, so the answer must not drift.
        let sandbox = Sandbox::new();
        let expected = canonical(&sandbox.cwd());
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("pwd", &expected);
        shell.expect_command("pwd", &expected);
    }

    #[test]
    fn pwd_reports_a_nested_start_directory() {
        // Guards against a `pwd` hardcoded to HOME or to the tempdir root.
        let sandbox = Sandbox::new();
        let nested = sandbox.mkdir("cwd/one/two/three");
        let mut shell = Session::spawn_in(&sandbox, &nested);
        shell.expect_prompt();
        shell.expect_command("pwd", &canonical(&nested));
    }
}

/// 02-navigation-02-ra6 — `cd` with absolute paths.
mod navigation_02_ra6 {
    use super::*;

    // --- spec ---

    #[test]
    fn cd_changes_to_an_absolute_path() {
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("elsewhere/deep");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cd {}", target.display()), "");
        shell.expect_command("pwd", &canonical(&target));
    }

    #[test]
    fn cd_to_a_missing_directory_reports_an_error() {
        let sandbox = Sandbox::new();
        let missing = sandbox.root().join("does_not_exist");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cd {}", missing.display()),
            &format!("cd: {}: No such file or directory", missing.display()),
        );
    }

    #[test]
    fn a_failed_cd_leaves_the_directory_unchanged() {
        let sandbox = Sandbox::new();
        let missing = sandbox.root().join("does_not_exist");
        let start = canonical(&sandbox.cwd());
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cd {}", missing.display()),
            &format!("cd: {}: No such file or directory", missing.display()),
        );
        shell.expect_command("pwd", &start);
    }

    // --- additional ---

    #[test]
    fn a_successful_cd_prints_nothing() {
        // The stage's transcript shows `$ cd /usr/local/bin` with no output
        // before the next prompt.
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("target");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cd {}", target.display()), "");
    }

    #[test]
    fn cd_can_be_repeated_between_directories() {
        let sandbox = Sandbox::new();
        let first = sandbox.mkdir("first");
        let second = sandbox.mkdir("second");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cd {}", first.display()), "");
        shell.expect_command("pwd", &canonical(&first));
        shell.expect_command(&format!("cd {}", second.display()), "");
        shell.expect_command("pwd", &canonical(&second));
    }

    #[test]
    fn the_shell_survives_a_failed_cd() {
        let sandbox = Sandbox::new();
        let missing = sandbox.root().join("nope");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cd {}", missing.display()),
            &format!("cd: {}: No such file or directory", missing.display()),
        );
        shell.expect_command("echo still here", "still here");
        shell.expect_alive();
    }

    #[test]
    fn cd_reports_the_path_as_given() {
        // The error message echoes the argument, so a shell that normalizes or
        // absolutizes the path before printing would differ from the tester.
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            "cd /definitely/not/here",
            "cd: /definitely/not/here: No such file or directory",
        );
    }
}

/// 02-navigation-03-gq9 — `cd` with relative paths.
mod navigation_03_gq9 {
    use super::*;

    // --- spec ---

    #[test]
    fn cd_follows_a_dot_slash_path() {
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("cwd/local/bin");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ./local/bin", "");
        shell.expect_command("pwd", &canonical(&target));
    }

    #[test]
    fn cd_follows_a_bare_relative_name() {
        // `dirname` is shorthand for `./dirname`.
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("cwd/local");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd local", "");
        shell.expect_command("pwd", &canonical(&target));
    }

    #[test]
    fn cd_walks_up_with_dot_dot() {
        let sandbox = Sandbox::new();
        let start = sandbox.mkdir("cwd/one/two");
        let mut shell = Session::spawn_in(&sandbox, &start);
        shell.expect_prompt();
        shell.expect_command("cd ../../", "");
        shell.expect_command("pwd", &canonical(&sandbox.cwd()));
    }

    #[test]
    fn cd_to_dot_stays_put() {
        let sandbox = Sandbox::new();
        let start = canonical(&sandbox.cwd());
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ./", "");
        shell.expect_command("pwd", &start);
    }

    #[test]
    fn a_missing_relative_path_reports_an_error() {
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ./missing", "cd: ./missing: No such file or directory");
    }

    #[test]
    fn a_failed_relative_cd_leaves_the_directory_unchanged() {
        let sandbox = Sandbox::new();
        let start = canonical(&sandbox.cwd());
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ./missing", "cd: ./missing: No such file or directory");
        shell.expect_command("pwd", &start);
    }

    // --- additional ---

    #[test]
    fn relative_paths_compose_across_commands() {
        // Each `cd` is resolved against the directory the previous one left.
        let sandbox = Sandbox::new();
        sandbox.mkdir("cwd/a/b/c");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd a", "");
        shell.expect_command("cd b", "");
        shell.expect_command("cd c", "");
        shell.expect_command("pwd", &canonical(&sandbox.cwd().join("a/b/c")));
    }

    #[test]
    fn dot_dot_and_descent_can_be_mixed() {
        let sandbox = Sandbox::new();
        sandbox.mkdir("cwd/left/inner");
        sandbox.mkdir("cwd/right");
        let start = sandbox.cwd().join("left/inner");
        let mut shell = Session::spawn_in(&sandbox, &start);
        shell.expect_prompt();
        shell.expect_command("cd ../../right", "");
        shell.expect_command("pwd", &canonical(&sandbox.cwd().join("right")));
    }

    #[test]
    fn a_trailing_slash_is_accepted() {
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("cwd/withslash");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd withslash/", "");
        shell.expect_command("pwd", &canonical(&target));
    }
}

/// 02-navigation-04-gp4 — `cd ~`.
mod navigation_04_gp4 {
    use super::*;

    // --- spec ---

    #[test]
    fn cd_tilde_goes_to_home() {
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &canonical(&sandbox.home()));
    }

    #[test]
    fn cd_tilde_uses_the_home_environment_variable() {
        // HOME points somewhere other than the default sandbox home, so a shell
        // that guesses the user's real home directory fails here.
        let mut sandbox = Sandbox::new();
        let custom = sandbox.mkdir("custom_home");
        let custom_str = custom.display().to_string();
        sandbox.set_env("HOME", &custom_str);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &canonical(&custom));
    }

    #[test]
    fn cd_tilde_works_from_an_unrelated_directory() {
        let sandbox = Sandbox::new();
        let elsewhere = sandbox.mkdir("elsewhere");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cd {}", elsewhere.display()), "");
        shell.expect_command("pwd", &canonical(&elsewhere));
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &canonical(&sandbox.home()));
    }

    // --- additional ---

    #[test]
    fn cd_tilde_prints_nothing_on_success() {
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ~", "");
    }

    #[test]
    fn cd_tilde_can_be_repeated() {
        let sandbox = Sandbox::new();
        let elsewhere = sandbox.mkdir("elsewhere");
        let home = canonical(&sandbox.home());
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &home);
        shell.expect_command(&format!("cd {}", elsewhere.display()), "");
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &home);
    }
}

/// The path as the operating system reports it.
///
/// On macOS the temp directory is reached through a `/var` -> `/private/var`
/// symlink, so the path handed to the shell and the path `pwd` prints are not
/// textually equal. Canonicalizing the expectation compares the directory the
/// shell actually landed in, which is what the stage is about.
fn canonical(path: &std::path::Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|err| panic!("failed to canonicalize {}: {err}", path.display()))
        .display()
        .to_string()
}
