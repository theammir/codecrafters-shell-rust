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
        shell.expect_command("pwd", &sandbox.cwd().display().to_string());
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
        let expected = sandbox.cwd().display().to_string();
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
        shell.expect_command("pwd", &nested.display().to_string());
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
        shell.expect_command("pwd", &target.display().to_string());
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
        let start = sandbox.cwd().display().to_string();
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
        shell.expect_command("pwd", &first.display().to_string());
        shell.expect_command(&format!("cd {}", second.display()), "");
        shell.expect_command("pwd", &second.display().to_string());
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
        shell.expect_command("pwd", &target.display().to_string());
    }

    #[test]
    fn cd_follows_a_bare_relative_name() {
        // `dirname` is shorthand for `./dirname`.
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("cwd/local");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd local", "");
        shell.expect_command("pwd", &target.display().to_string());
    }

    #[test]
    fn cd_walks_up_with_dot_dot() {
        let sandbox = Sandbox::new();
        let start = sandbox.mkdir("cwd/one/two");
        let mut shell = Session::spawn_in(&sandbox, &start);
        shell.expect_prompt();
        shell.expect_command("cd ../../", "");
        shell.expect_command("pwd", &sandbox.cwd().display().to_string());
    }

    #[test]
    fn cd_to_dot_stays_put() {
        let sandbox = Sandbox::new();
        let start = sandbox.cwd().display().to_string();
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
        let start = sandbox.cwd().display().to_string();
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
        shell.expect_command("pwd", &sandbox.cwd().join("a/b/c").display().to_string());
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
        shell.expect_command("pwd", &sandbox.cwd().join("right").display().to_string());
    }

    #[test]
    fn a_trailing_slash_is_accepted() {
        let sandbox = Sandbox::new();
        let target = sandbox.mkdir("cwd/withslash");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd withslash/", "");
        shell.expect_command("pwd", &target.display().to_string());
    }

    #[test]
    fn cd_into_a_symlinked_directory_lands_in_that_directory() {
        // Symlinked directories are everywhere in practice. Shells differ in
        // whether `pwd` then reports the link path or the resolved path (bash
        // tracks a logical PWD; a shell built on getcwd reports the physical
        // one), and no stage picks a side, so this asserts only that the shell
        // ended up in the right directory by either name.
        let sandbox = Sandbox::new();
        let real = sandbox.mkdir("cwd/real_target");
        let link = sandbox.symlink(&real, "cwd/link_to_target");

        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd link_to_target", "");
        shell.send_line("pwd");
        shell.read_until("\n");
        let reported = shell.read_until_prompt();

        assert_eq!(
            std::fs::canonicalize(&reported).ok(),
            std::fs::canonicalize(&real).ok(),
            "pwd reported {reported:?}, which is neither {} nor {}",
            link.display(),
            real.display()
        );
    }

    #[test]
    fn a_symlinked_directory_can_be_left_again() {
        // Whichever convention the shell uses for `pwd`, leaving a symlinked
        // directory must reach a real directory and not strand the shell.
        let sandbox = Sandbox::new();
        let real = sandbox.mkdir("cwd/real_target");
        sandbox.symlink(&real, "cwd/link_to_target");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd link_to_target", "");
        shell.expect_command("cd ..", "");
        shell.send_line("pwd");
        shell.read_until("\n");
        let reported = shell.read_until_prompt();
        assert!(
            std::fs::canonicalize(&reported).is_ok(),
            "pwd reported {reported:?}, which is not an existing directory"
        );
        shell.expect_alive();
    }

    #[test]
    fn cd_through_a_symlink_to_a_missing_target_reports_an_error() {
        // A dangling symlink is not a directory, so `cd` must fail the same way
        // it fails for a name that does not exist at all.
        let sandbox = Sandbox::new();
        sandbox.symlink(sandbox.root().join("no_such_dir"), "cwd/broken_link");
        let start = sandbox.cwd().display().to_string();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            "cd broken_link",
            "cd: broken_link: No such file or directory",
        );
        shell.expect_command("pwd", &start);
        shell.expect_alive();
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
        shell.expect_command("pwd", &sandbox.home().display().to_string());
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
        shell.expect_command("pwd", &custom.display().to_string());
    }

    #[test]
    fn cd_tilde_works_from_an_unrelated_directory() {
        let sandbox = Sandbox::new();
        let elsewhere = sandbox.mkdir("elsewhere");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cd {}", elsewhere.display()), "");
        shell.expect_command("pwd", &elsewhere.display().to_string());
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &sandbox.home().display().to_string());
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
        let home = sandbox.home().display().to_string();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &home);
        shell.expect_command(&format!("cd {}", elsewhere.display()), "");
        shell.expect_command("cd ~", "");
        shell.expect_command("pwd", &home);
    }
}
