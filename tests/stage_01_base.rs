//! Stage 1: prompt, REPL, `exit`, `echo`, `type`, external programs.

mod harness;

use harness::{Sandbox, Session};

/// 01-base-01-oo8 — print a prompt.
mod base_01_oo8 {
    use super::*;

    // --- spec ---

    #[test]
    fn prints_the_prompt() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
    }

    // --- additional ---

    #[test]
    fn prompt_appears_before_any_input_is_sent() {
        // The prompt must be flushed, not merely buffered: the tester reads it
        // before writing anything, so an unflushed prompt looks like a hang.
        let (mut shell, _sandbox) = Session::new();
        let before_prompt = shell.read_until("$ ");
        assert_eq!(
            before_prompt, "",
            "expected nothing before the prompt, got {before_prompt:?}"
        );
    }
}

/// 01-base-02-cz2 — report invalid commands.
mod base_02_cz2 {
    use super::*;

    // --- spec ---

    #[test]
    fn reports_unknown_command() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("xyz", "xyz: command not found");
    }

    // --- additional ---

    #[test]
    fn reports_the_command_name_verbatim() {
        // The message echoes the name as typed; a shell that lowercases or
        // trims differently would pass the simple case and fail here.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("Weird_Name123", "Weird_Name123: command not found");
    }

    #[test]
    fn reports_a_command_that_has_arguments() {
        // Only the command name appears in the message, not the arguments.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("xyz foo bar", "xyz: command not found");
    }
}

/// 01-base-03-ff0 — loop indefinitely.
mod base_03_ff0 {
    use super::*;

    // --- spec ---

    #[test]
    fn loops_over_several_invalid_commands() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        for name in [
            "invalid_command_1",
            "invalid_command_2",
            "invalid_command_3",
        ] {
            shell.expect_command(name, &format!("{name}: command not found"));
        }
        shell.expect_alive();
    }

    // --- additional ---

    #[test]
    fn prompts_again_after_each_command() {
        // expect_command consumes the following prompt, so reaching the last
        // assertion at all proves a prompt was printed between each command.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("aaa", "aaa: command not found");
        shell.expect_command("bbb", "bbb: command not found");
        shell.expect_alive();
    }

    #[test]
    fn survives_a_long_run_of_commands() {
        // Guards against state that accumulates across iterations, e.g. an
        // input buffer that is never cleared.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        for i in 0..25 {
            let name = format!("cmd_{i}");
            shell.expect_command(&name, &format!("{name}: command not found"));
        }
        shell.expect_alive();
    }

    #[test]
    fn repeating_one_command_keeps_reporting_it() {
        // The same input twice must produce the same output twice.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("same_command", "same_command: command not found");
        shell.expect_command("same_command", "same_command: command not found");
    }
}

/// 01-base-04-pn5 — the `exit` builtin.
mod base_04_pn5 {
    use super::*;

    // --- spec ---

    #[test]
    fn exit_terminates_the_shell() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.send_line("exit");
        shell.expect_exit();
    }

    #[test]
    fn exit_terminates_after_an_invalid_command() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("invalid_command_1", "invalid_command_1: command not found");
        shell.send_line("exit");
        shell.expect_exit();
    }

    // --- additional ---

    #[test]
    fn exit_produces_no_output() {
        // The stage shows `$ exit` with nothing after it. Anything printed here
        // would appear in the tester's diff.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.send_line("exit");
        shell.expect_exit();
        let tail = shell
            .transcript()
            .rsplit_once("exit")
            .map(|(_, after)| after.replace(['\r', '\n'], ""))
            .unwrap_or_default();
        assert_eq!(tail, "", "expected no output after `exit`, got {tail:?}");
    }
}

/// 01-base-05-iz3 — the `echo` builtin.
mod base_05_iz3 {
    use super::*;

    // --- spec ---

    #[test]
    fn echo_prints_its_arguments() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo hello world", "hello world");
        shell.expect_command("echo pineapple strawberry", "pineapple strawberry");
    }

    #[test]
    fn echo_prints_three_arguments() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo one two three", "one two three");
    }

    // --- additional ---

    #[test]
    fn echo_with_a_single_argument() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo solo", "solo");
    }

    #[test]
    fn echo_with_no_arguments_prints_an_empty_line() {
        // The stage defines echo as printing its arguments joined by spaces
        // followed by a newline. With no arguments that is just the newline.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo", "");
    }

    #[test]
    fn shell_survives_a_sequence_of_echoes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        for word in ["alpha", "beta", "gamma"] {
            shell.expect_command(&format!("echo {word}"), word);
        }
        shell.expect_alive();
    }
}

/// 01-base-06-ez5 — `type` for builtins.
mod base_06_ez5 {
    use super::*;

    // --- spec ---

    #[test]
    fn type_reports_echo_as_a_builtin() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type echo", "echo is a shell builtin");
    }

    #[test]
    fn type_reports_exit_as_a_builtin() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type exit", "exit is a shell builtin");
    }

    #[test]
    fn type_reports_itself_as_a_builtin() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type type", "type is a shell builtin");
    }

    #[test]
    fn type_reports_an_unknown_command_as_not_found() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type invalid_command", "invalid_command: not found");
    }

    #[test]
    fn type_handles_a_sequence_of_queries() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type echo", "echo is a shell builtin");
        shell.expect_command("type exit", "exit is a shell builtin");
        shell.expect_command("type type", "type is a shell builtin");
        shell.expect_command("type invalid_command", "invalid_command: not found");
        shell.expect_alive();
    }

    // --- additional ---

    #[test]
    fn not_found_message_differs_from_the_command_not_found_message() {
        // `type xyz` reports `xyz: not found`, while running `xyz` reports
        // `xyz: command not found`. Reusing one message for both is an easy
        // mistake and the tester compares exactly.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type xyz", "xyz: not found");
        shell.expect_command("xyz", "xyz: command not found");
    }

    #[test]
    fn type_reports_the_queried_name_verbatim() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("type Weird_Name123", "Weird_Name123: not found");
    }
}

/// 01-base-07-mg5 — `type` searches PATH.
mod base_07_mg5 {
    use super::*;

    // --- spec ---

    #[test]
    fn type_reports_an_executable_with_its_full_path() {
        let sandbox = Sandbox::new();
        let path = sandbox.install_executable("valid_command", "echo ran");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            "type valid_command",
            &format!("valid_command is {}", path.display()),
        );
    }

    #[test]
    fn type_still_reports_builtins_as_builtins() {
        // Builtins take precedence: the lookup must not fall through to PATH
        // even when a same-named executable exists there.
        let sandbox = Sandbox::new();
        sandbox.install_executable("echo", "echo impostor");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type echo", "echo is a shell builtin");
    }

    #[test]
    fn type_reports_not_found_when_no_executable_exists() {
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type invalid_command", "invalid_command: not found");
    }

    #[test]
    fn type_skips_files_without_execute_permission() {
        // The stage is explicit: a file that exists but lacks execute
        // permission is skipped, and the search continues.
        let sandbox = Sandbox::new();
        sandbox.install_non_executable("tool");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type tool", "tool: not found");
    }

    #[test]
    fn type_continues_past_a_non_executable_into_a_later_directory() {
        // A non-executable `tool` early on PATH must not mask the real one in a
        // later directory.
        let mut sandbox = Sandbox::new();
        sandbox.install_non_executable("tool");
        let second = sandbox.mkdir("bin2");
        let real = sandbox.install_executable_in(&second, "tool", "echo real");
        sandbox.push_path_dir(&second);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type tool", &format!("tool is {}", real.display()));
    }

    // --- additional ---

    #[test]
    fn type_uses_the_first_match_on_path() {
        // PATH is searched left to right, so the earlier directory wins.
        let mut sandbox = Sandbox::new();
        let first = sandbox.install_executable("tool", "echo first");
        let second_dir = sandbox.mkdir("bin2");
        sandbox.install_executable_in(&second_dir, "tool", "echo second");
        sandbox.push_path_dir(&second_dir);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type tool", &format!("tool is {}", first.display()));
    }

    #[test]
    fn type_tolerates_a_nonexistent_directory_on_path() {
        // The stage notes PATH can name directories that do not exist; the
        // search must step over them rather than fail.
        let mut sandbox = Sandbox::new();
        let missing = sandbox.root().join("does_not_exist");
        let real = sandbox.install_executable("tool", "echo real");
        // Prepend the missing directory by rebuilding PATH.
        let bin = sandbox.bin();
        sandbox.set_env("PATH", &format!("{}:{}", missing.display(), bin.display()));
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type tool", &format!("tool is {}", real.display()));
    }

    #[test]
    fn type_finds_an_executable_in_a_directory_with_a_trailing_component() {
        // A second PATH entry that is a subdirectory, to confirm each entry is
        // joined with the command name rather than concatenated.
        let mut sandbox = Sandbox::new();
        let nested = sandbox.mkdir("nested/deeper");
        let path = sandbox.install_executable_in(&nested, "nested_tool", "echo ran");
        sandbox.push_path_dir(&nested);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            "type nested_tool",
            &format!("nested_tool is {}", path.display()),
        );
    }

    #[test]
    fn type_reports_a_usable_path_when_the_path_entry_is_a_symlink() {
        // Symlinked PATH entries are ordinary: /usr/bin is reached through one
        // on macOS, and a Nix profile's bin is a symlink into the store.
        //
        // The stage says to print `<command> is <full_path>` without saying
        // whether symlinks are resolved, so both the link path and the resolved
        // path satisfy it. This asserts only what the stage does pin down: the
        // reported path must be absolute and must actually be the executable.
        let mut sandbox = Sandbox::new();
        let real_dir = sandbox.mkdir("real/bin");
        sandbox.install_executable_in(&real_dir, "linked_tool", "echo ran");
        let link_dir = sandbox.symlink(&real_dir, "linkbin");
        sandbox.push_path_dir(&link_dir);

        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.send_line("type linked_tool");
        shell.read_until("\n");
        let output = shell.read_until_prompt();

        let reported = output
            .strip_prefix("linked_tool is ")
            .unwrap_or_else(|| panic!("expected `linked_tool is <path>`, got {output:?}"));
        assert_eq!(
            std::fs::canonicalize(reported).ok(),
            std::fs::canonicalize(real_dir.join("linked_tool")).ok(),
            "reported path {reported:?} does not resolve to the installed executable"
        );
    }

    #[test]
    fn type_reports_a_usable_path_when_the_executable_is_a_symlink() {
        // The PATH entry is a real directory, but the executable inside it is a
        // symlink to a file elsewhere.
        let sandbox = Sandbox::new();
        let store = sandbox.mkdir("store");
        let target = sandbox.install_executable_in(&store, "actual_tool", "echo ran");
        sandbox.symlink(&target, "bin/aliased_tool");

        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.send_line("type aliased_tool");
        shell.read_until("\n");
        let output = shell.read_until_prompt();

        let reported = output
            .strip_prefix("aliased_tool is ")
            .unwrap_or_else(|| panic!("expected `aliased_tool is <path>`, got {output:?}"));
        assert_eq!(
            std::fs::canonicalize(reported).ok(),
            std::fs::canonicalize(&target).ok(),
            "reported path {reported:?} does not resolve to the installed executable"
        );
    }

    #[test]
    fn a_dangling_symlink_on_path_is_not_reported_as_found() {
        // A symlink whose target does not exist is not an executable file, so
        // the search must step over it exactly as it steps over a missing
        // directory. A shell that resolves paths without handling the failure
        // would crash here.
        let sandbox = Sandbox::new();
        sandbox.symlink(sandbox.root().join("no_such_target"), "bin/broken_tool");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type broken_tool", "broken_tool: not found");
        shell.expect_alive();
    }

    #[test]
    fn a_dangling_symlink_does_not_mask_a_later_executable() {
        let mut sandbox = Sandbox::new();
        sandbox.symlink(sandbox.root().join("no_such_target"), "bin/shadowed");
        let second = sandbox.mkdir("bin2");
        let real = sandbox.install_executable_in(&second, "shadowed", "echo real");
        sandbox.push_path_dir(&second);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("type shadowed", &format!("shadowed is {}", real.display()));
    }
}

/// 01-base-08-ip1 — run external programs.
mod base_08_ip1 {
    use super::*;

    // --- spec ---

    #[test]
    fn runs_an_external_program() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("custom_exe_1234", "echo ran the program");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("custom_exe_1234", "ran the program");
    }

    #[test]
    fn passes_arguments_to_the_program() {
        // The program receives its own name as argument 0 and the rest after,
        // which is what the stage's sample output reports.
        //
        // The fixture uses only shell builtins and `${0##*/}` rather than
        // `basename`: the sandbox PATH holds nothing but the fake executables,
        // so any external command the script calls is itself not found.
        //
        // argv[0] is stripped to its last component because the kernel rewrites
        // it to the full script path when running a `#!` file, whatever the
        // shell passed. Comparing the basename keeps the assertion about the
        // shell's argument passing rather than about shebang mechanics.
        let sandbox = Sandbox::new();
        sandbox.install_executable(
            "custom_exe_1234",
            r#"echo "Program was passed $(($# + 1)) args (including program name)."
echo "Arg #0 (program name): ${0##*/}"
i=1
for arg in "$@"; do
  echo "Arg #$i: $arg"
  i=$((i + 1))
done"#,
        );
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            "custom_exe_1234 alice",
            "Program was passed 2 args (including program name).\n\
             Arg #0 (program name): custom_exe_1234\n\
             Arg #1: alice",
        );
    }

    #[test]
    fn passes_several_arguments_in_order() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("show_args", r#"for arg in "$@"; do echo "$arg"; done"#);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("show_args one two three", "one\ntwo\nthree");
    }

    // --- additional ---

    #[test]
    fn program_output_appears_before_the_next_prompt() {
        // The shell must wait for the child to finish before prompting again,
        // otherwise output interleaves with the prompt.
        let sandbox = Sandbox::new();
        sandbox.install_executable("slow_exe", "echo first line\necho second line");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("slow_exe", "first line\nsecond line");
        shell.expect_alive();
    }

    #[test]
    fn unknown_command_still_reports_not_found() {
        // Adding PATH lookup must not lose the error path for names that are
        // genuinely absent.
        let sandbox = Sandbox::new();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("no_such_program", "no_such_program: command not found");
    }

    #[test]
    fn a_non_executable_file_on_path_is_not_run() {
        // Same permission rule as `type`: a non-executable match is skipped, so
        // the command counts as not found.
        let sandbox = Sandbox::new();
        sandbox.install_non_executable("inert");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("inert", "inert: command not found");
    }

    #[test]
    fn an_executable_behind_a_symlinked_path_entry_can_be_run() {
        // A symlinked PATH entry is ordinary: /usr/bin is reached through one
        // on macOS, and a Nix profile's bin is a symlink into the store. The
        // program must be found and run through it.
        let mut sandbox = Sandbox::new();
        let real_dir = sandbox.mkdir("real/bin");
        sandbox.install_executable_in(&real_dir, "linked_tool", "echo ran through link");
        let link_dir = sandbox.symlink(&real_dir, "linkbin");
        sandbox.push_path_dir(&link_dir);
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("linked_tool", "ran through link");
    }

    #[test]
    fn an_executable_that_is_itself_a_symlink_can_be_run() {
        // The PATH entry is a real directory holding a symlink to the program.
        let sandbox = Sandbox::new();
        let store = sandbox.mkdir("store");
        let target = sandbox.install_executable_in(&store, "actual_tool", "echo ran via alias");
        sandbox.symlink(&target, "bin/aliased_tool");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("aliased_tool", "ran via alias");
    }

    #[test]
    fn a_dangling_symlink_on_path_does_not_stop_the_shell() {
        // A symlink with no target is not runnable, so the command is not
        // found — and the shell must survive to serve the next prompt.
        let sandbox = Sandbox::new();
        sandbox.symlink(sandbox.root().join("no_such_target"), "bin/broken_tool");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("broken_tool", "broken_tool: command not found");
        shell.expect_alive();
    }

    #[test]
    fn shell_survives_running_several_programs() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("prog_a", "echo a");
        sandbox.install_executable("prog_b", "echo b");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("prog_a", "a");
        shell.expect_command("prog_b", "b");
        shell.expect_command("prog_a", "a");
        shell.expect_alive();
    }

    #[test]
    fn builtins_still_work_after_running_a_program() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("prog", "echo from program");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("prog", "from program");
        shell.expect_command("echo from builtin", "from builtin");
        shell.expect_command(
            "type prog",
            &format!("prog is {}", sandbox.bin().join("prog").display()),
        );
    }
}
