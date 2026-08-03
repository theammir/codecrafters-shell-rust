//! Stage 3: single quotes, double quotes, and backslash escaping.
//!
//! Several steps exercise the parser through `cat`, as the stage descriptions
//! do: a filename that survives quoting and reaches `exec` intact proves the
//! argument was parsed correctly, in a way `echo` alone cannot.

mod harness;

use harness::{Sandbox, Session};

/// 03-quoting-01-ni6 — single quotes.
mod quoting_01_ni6 {
    use super::*;

    // --- spec ---

    #[test]
    fn single_quotes_preserve_spaces() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'shell hello'", "shell hello");
        shell.expect_command("echo 'world     test'", "world     test");
    }

    #[test]
    fn unquoted_consecutive_spaces_are_collapsed() {
        // The counterpart to the rule above: without quotes, runs of spaces are
        // delimiters, so `echo` sees two arguments and joins them with one space.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo hello    world", "hello world");
    }

    #[test]
    fn adjacent_quoted_strings_are_concatenated() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'hello''world'", "helloworld");
    }

    #[test]
    fn empty_quotes_are_ignored() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo hello''world", "helloworld");
    }

    #[test]
    fn cat_reads_files_whose_names_are_single_quoted() {
        // The stage sends exactly this: two quoted paths containing spaces.
        // `cat` writes each file's contents with no separator, so the shell
        // must pass two arguments with their spaces intact.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/file name", "content1 ");
        sandbox.write_file("files/file name with spaces", "content2");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!(
                "cat '{0}/file name' '{0}/file name with spaces'",
                dir.display()
            ),
            "content1 content2",
        );
    }

    // --- additional ---

    #[test]
    fn special_characters_lose_their_meaning_inside_single_quotes() {
        // The stage lists `$`, `*` and `~` as characters that become literal.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo '$HOME'", "$HOME");
        shell.expect_command("echo '*'", "*");
        shell.expect_command("echo '~'", "~");
    }

    #[test]
    fn double_quotes_are_literal_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a\"b'", "a\"b");
    }

    #[test]
    fn a_lone_pair_of_quotes_produces_an_empty_argument() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo ''", "");
    }

    #[test]
    fn quoted_arguments_are_still_separated_by_unquoted_spaces() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a  b'   'c  d'", "a  b c  d");
    }

    #[test]
    fn the_shell_survives_a_sequence_of_quoted_commands() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'one'", "one");
        shell.expect_command("echo 'two  two'", "two  two");
        shell.expect_command("echo 'three'", "three");
        shell.expect_alive();
    }
}

/// 03-quoting-02-tg6 — double quotes.
mod quoting_02_tg6 {
    use super::*;

    // --- spec ---

    #[test]
    fn double_quotes_preserve_spaces() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"quz  hello\"  \"bar\"", "quz  hello bar");
    }

    #[test]
    fn single_quotes_are_literal_inside_double_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"bar\"  \"shell's\"  \"foo\"", "bar shell's foo");
    }

    #[test]
    fn adjacent_double_quoted_strings_are_concatenated() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"hello\"\"world\"", "helloworld");
    }

    #[test]
    fn quoted_and_unquoted_strings_are_concatenated() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"hello\"world", "helloworld");
    }

    #[test]
    fn separately_quoted_strings_stay_separate_arguments() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"hello\" \"world\"", "hello world");
    }

    #[test]
    fn cat_reads_files_whose_names_are_double_quoted() {
        // The stage's second filename contains single quotes, which must be
        // literal inside double quotes.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/file name", "content1 ");
        sandbox.write_file("files/'file name' with spaces", "content2");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!(
                "cat \"{0}/file name\" \"{0}/'file name' with spaces\"",
                dir.display()
            ),
            "content1 content2",
        );
    }

    // --- additional ---

    #[test]
    fn double_quotes_preserve_a_long_run_of_spaces() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"hello    world\"", "hello    world");
    }

    #[test]
    fn an_empty_double_quoted_string_produces_an_empty_argument() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"\"", "");
    }

    #[test]
    fn the_two_quote_styles_can_be_mixed_across_arguments() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a  b' \"c  d\"", "a  b c  d");
    }

    #[test]
    fn the_two_quote_styles_concatenate_within_one_argument() {
        // Both quoting stages state that adjacent quoted and unquoted strings
        // concatenate; neither restricts that to a single quote style.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"one\"'two'three''four", "onetwothreefour");
        shell.expect_command("echo 'one'\"two\"", "onetwo");
        shell.expect_command("echo one\"two\"'three'", "onetwothree");
    }

    #[test]
    fn empty_quotes_of_either_style_vanish_between_words() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo a\"\"b", "ab");
        shell.expect_command("echo a''\"\"b", "ab");
    }

    #[test]
    fn concatenation_preserves_spaces_from_every_piece() {
        // The joined pieces keep their own internal spacing, and the result is
        // still one argument: the unquoted `c` cannot be split off.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a  b'\"  c  \"d", "a  b  c  d");
    }

    #[test]
    fn mixed_quote_concatenation_builds_a_single_filename() {
        // Proven through `cat`: only one argument can name this file, so the
        // three differently quoted pieces must have become one word.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/mixed 'name' here", "content1");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cat '{}/mixed '\"'name'\"' here'", dir.display()),
            "content1",
        );
    }

    #[test]
    fn each_quote_style_is_literal_inside_the_other_when_concatenated() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"it's\"'a \"test\"'", "it'sa \"test\"");
    }

    #[test]
    fn a_quoted_argument_can_follow_an_unquoted_one() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo plain \"quoted  arg\"", "plain quoted  arg");
    }
}

/// 03-quoting-03-yt5 — backslashes outside quotes.
mod quoting_03_yt5 {
    use super::*;

    // --- spec ---

    #[test]
    fn backslash_escapes_a_space_into_one_argument() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo three\\ \\ \\ spaces", "three   spaces");
    }

    #[test]
    fn backslash_escapes_multiple_spaces() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo multiple\\ \\ \\ \\ spaces", "multiple    spaces");
    }

    #[test]
    fn an_escaped_space_survives_a_following_collapsed_run() {
        // From the stage table: the escaped space is kept, and the unescaped
        // run after it collapses to a single separator, giving two spaces.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo before\\     after", "before  after");
    }

    #[test]
    fn backslash_before_a_plain_letter_is_dropped() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo test\\nexample", "testnexample");
        shell.expect_command("echo ignore\\_backslash", "ignore_backslash");
    }

    #[test]
    fn a_doubled_backslash_becomes_one_literal_backslash() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo hello\\\\world", "hello\\world");
    }

    #[test]
    fn backslash_makes_quotes_literal() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \\'hello\\'", "'hello'");
        shell.expect_command("echo \\'\\\"literal quotes\\\"\\'", "'\"literal quotes\"'");
    }

    #[test]
    fn cat_reads_files_whose_names_use_backslash_escapes() {
        // Mirrors the stage's own example: an escaped leading character, an
        // escaped digit, and an escaped backslash inside a filename.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/_ignored_1", "content1 ");
        sandbox.write_file("files/ignore_2", "content2 ");
        sandbox.write_file("files/just_one_\\_3", "content3");
        let base = dir.display().to_string();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cat {base}/\\_ignored_1 {base}/ignore_\\2 {base}/just_one_\\\\_3"),
            "content1 content2 content3",
        );
    }

    // --- additional ---

    #[test]
    fn a_backslash_escaped_space_keeps_one_argument_together() {
        // Proven through the filesystem rather than `echo`: only a single
        // argument can name this file.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/one file", "joined");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(&format!("cat {}/one\\ file", dir.display()), "joined");
    }

    #[test]
    fn backslash_escapes_a_double_quote_outside_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \\\"quoted\\\"", "\"quoted\"");
    }

    #[test]
    fn the_shell_survives_a_sequence_of_escaped_commands() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo a\\ b", "a b");
        shell.expect_command("echo c\\\\d", "c\\d");
        shell.expect_command("echo e\\'f", "e'f");
        shell.expect_alive();
    }
}

/// 03-quoting-04-le5 — backslashes inside single quotes.
mod quoting_04_le5 {
    use super::*;

    // --- spec ---

    #[test]
    fn backslashes_are_literal_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'shell\\\\\\nscript'", "shell\\\\\\nscript");
        shell.expect_command("echo 'example\\\"test'", "example\\\"test");
    }

    #[test]
    fn doubled_backslashes_stay_doubled_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'multiple\\\\slashes'", "multiple\\\\slashes");
    }

    #[test]
    fn escaped_quotes_stay_literal_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command(
            "echo 'every\\\"thing_is\\\"literal'",
            "every\\\"thing_is\\\"literal",
        );
    }

    #[test]
    fn cat_reads_files_named_with_backslashes_in_single_quotes() {
        // The stage's example: filenames containing literal backslashes, given
        // inside single quotes where no escaping happens.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/no slash 1", "content1 ");
        sandbox.write_file("files/one slash \\2", "content2 ");
        sandbox.write_file("files/two slashes \\3\\", "content3");
        let base = dir.display().to_string();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!("cat {base}/'no slash 1' {base}/'one slash \\2' {base}/'two slashes \\3\\'"),
            "content1 content2 content3",
        );
    }

    // --- additional ---

    #[test]
    fn a_single_backslash_is_literal_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a\\b'", "a\\b");
    }

    #[test]
    fn a_trailing_backslash_is_literal_inside_single_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'ends\\'", "ends\\");
    }

    #[test]
    fn a_backslash_cannot_escape_the_closing_single_quote() {
        // Since backslashes are literal inside single quotes, the quote after
        // one still closes the string, and the following word is separate.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo 'a\\' b", "a\\ b");
    }
}

/// 03-quoting-05-gu3 — backslashes inside double quotes.
mod quoting_05_gu3 {
    use super::*;

    // --- spec ---

    #[test]
    fn backslash_escapes_a_backslash_inside_double_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"A \\\\ escapes itself\"", "A \\ escapes itself");
    }

    #[test]
    fn backslash_escapes_a_double_quote_inside_double_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command(
            "echo \"A \\\" inside double quotes\"",
            "A \" inside double quotes",
        );
    }

    #[test]
    fn a_doubled_backslash_collapses_inside_double_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command(
            "echo \"just'one'\\\\n'backslash\"",
            "just'one'\\n'backslash",
        );
    }

    #[test]
    fn escaped_quotes_can_open_and_close_around_a_word() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command(
            "echo \"inside\\\"literal_quote.\"outside\\\"",
            "inside\"literal_quote.outside\"",
        );
    }

    #[test]
    fn cat_reads_files_named_with_backslashes_in_double_quotes() {
        // The stage's example: a filename containing a double quote and one
        // containing a backslash, both escaped inside double quotes.
        let sandbox = Sandbox::new();
        sandbox.install_system_executable("cat");
        let dir = sandbox.mkdir("files");
        sandbox.write_file("files/number 1", "content1 ");
        sandbox.write_file("files/doublequote \" 2", "content2 ");
        sandbox.write_file("files/backslash \\ 3", "content3");
        let base = dir.display().to_string();
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command(
            &format!(
                "cat {base}/\"number 1\" {base}/\"doublequote \\\" 2\" {base}/\"backslash \\\\ 3\""
            ),
            "content1 content2 content3",
        );
    }

    // --- additional ---

    #[test]
    fn a_backslash_before_a_plain_letter_stays_literal_in_double_quotes() {
        // The stage is explicit that only `"`, `\`, `$`, backtick and newline
        // are escapable here; before anything else the backslash is kept.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"before\\after\"", "before\\after");
    }

    #[test]
    fn a_backslash_before_a_single_quote_stays_literal_in_double_quotes() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"a\\'b\"", "a\\'b");
    }

    #[test]
    fn escaped_backslashes_can_repeat() {
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("echo \"back\\\\slash\"", "back\\slash");
    }
}

/// 03-quoting-06-qj0 — executing a quoted executable.
mod quoting_06_qj0 {
    use super::*;

    // --- spec ---

    #[test]
    fn runs_an_executable_whose_quoted_name_contains_double_quotes() {
        // The stage renames an executable to a name containing quotes and runs
        // it by single-quoting the whole name.
        let sandbox = Sandbox::new();
        sandbox.install_executable("exe with \"quotes\"", "echo content1");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("'exe with \"quotes\"' file", "content1");
    }

    #[test]
    fn runs_an_executable_whose_quoted_name_contains_single_quotes() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("exe with 'single quotes'", "echo content2");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("\"exe with 'single quotes'\" file", "content2");
    }

    #[test]
    fn arguments_after_a_quoted_executable_are_passed_through() {
        let sandbox = Sandbox::new();
        sandbox.install_executable(
            "exe with spaces",
            r#"for arg in "$@"; do echo "$arg"; done"#,
        );
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("'exe with spaces' one two", "one\ntwo");
    }

    // --- additional ---

    #[test]
    fn a_quoted_executable_name_without_special_characters_still_runs() {
        // Quoting a plain name changes nothing about which program is found.
        let sandbox = Sandbox::new();
        sandbox.install_executable("plain_exe", "echo ran");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("'plain_exe'", "ran");
        shell.expect_command("\"plain_exe\"", "ran");
    }

    #[test]
    fn a_quoted_builtin_name_is_still_the_builtin() {
        // Quote removal happens before the name is looked up, so the builtin is
        // found exactly as it is when unquoted.
        let (mut shell, _sandbox) = Session::new();
        shell.expect_prompt();
        shell.expect_command("'echo' hello", "hello");
        shell.expect_command("\"echo\" hello", "hello");
    }

    #[test]
    fn the_shell_survives_running_a_quoted_executable() {
        let sandbox = Sandbox::new();
        sandbox.install_executable("exe with spaces", "echo ok");
        let mut shell = Session::spawn(&sandbox);
        shell.expect_prompt();
        shell.expect_command("'exe with spaces'", "ok");
        shell.expect_command("echo after", "after");
        shell.expect_alive();
    }
}
