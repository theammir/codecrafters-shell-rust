//! PTY-backed shell session: spawn the binary on a terminal, send input, read output.
//!
//! Everything runs on a pseudo-terminal rather than pipes, because later stages
//! (tab completion, job control, history navigation) only behave correctly when
//! stdin is a TTY — readline implementations disable line editing on a pipe.

use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};

use super::sandbox::Sandbox;

/// Path to the shell under test. Cargo rebuilds it before the test binary runs,
/// so this always points at current `src/`.
pub const SHELL_BIN: &str = env!("CARGO_BIN_EXE_codecrafters-shell");

/// The prompt every stage expects.
pub const PROMPT: &str = "$ ";

/// How long any single read waits before declaring the shell hung.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// A running shell attached to a PTY.
pub struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    output: Receiver<Vec<u8>>,
    /// Everything read from the PTY so far, including bytes already consumed by
    /// an `expect_*` call. Kept whole so failures can show full context.
    seen: String,
    /// Offset into `seen` that matching starts from.
    cursor: usize,
    timeout: Duration,
}

impl Session {
    /// Spawn the shell inside `sandbox`, with only the sandbox's environment.
    pub fn spawn(sandbox: &Sandbox) -> Self {
        let cwd = sandbox.cwd();
        Self::spawn_in(sandbox, &cwd)
    }

    /// Spawn the shell in an explicit working directory, for tests that need to
    /// start somewhere other than the sandbox cwd.
    pub fn spawn_in(sandbox: &Sandbox, cwd: &std::path::Path) -> Self {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open pty");

        let mut cmd = CommandBuilder::new(SHELL_BIN);
        cmd.cwd(cwd);
        cmd.env_clear();
        for (key, value) in sandbox.env() {
            cmd.env(key, value);
        }

        let child = pty.slave.spawn_command(cmd).expect("failed to spawn shell");
        // Drop the slave handle: while the test process holds it open, reads on
        // the master never see EOF and a dead shell looks identical to a hung one.
        drop(pty.slave);

        let mut reader = pty
            .master
            .try_clone_reader()
            .expect("failed to clone reader");
        let writer = pty.master.take_writer().expect("failed to take writer");

        // A reader thread turns blocking PTY reads into something we can apply a
        // timeout to. It ends when the shell exits and the master reports EOF.
        let (tx, output) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });

        Self {
            master: pty.master,
            writer,
            child,
            output,
            seen: String::new(),
            cursor: 0,
            timeout: READ_TIMEOUT,
        }
    }

    /// Spawn with a fresh default sandbox. Convenience for tests that do not
    /// need to prepare the filesystem first.
    pub fn new() -> (Self, Sandbox) {
        let sandbox = Sandbox::new();
        let session = Self::spawn(&sandbox);
        (session, sandbox)
    }

    /// Override the per-read timeout. Use sparingly; a long timeout turns a hang
    /// into a slow suite.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Write raw bytes, no newline appended. For control characters like `\t`.
    pub fn send_raw(&mut self, bytes: &str) {
        self.writer
            .write_all(bytes.as_bytes())
            .expect("failed to write to shell");
        self.writer.flush().expect("failed to flush to shell");
    }

    /// Send one line of input, as a user pressing Return would.
    pub fn send_line(&mut self, line: &str) {
        self.send_raw(&format!("{line}\n"));
    }

    /// Pull whatever output has arrived, blocking until `deadline` at the latest.
    /// Returns false once the PTY is at EOF and no more output can arrive.
    fn pump(&mut self, deadline: Instant) -> bool {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match self.output.recv_timeout(remaining) {
            Ok(chunk) => {
                self.seen.push_str(&String::from_utf8_lossy(&chunk));
                true
            }
            Err(RecvTimeoutError::Timeout) => true,
            Err(RecvTimeoutError::Disconnected) => false,
        }
    }

    /// Read until `needle` appears, then consume through it and return
    /// everything that preceded it.
    ///
    /// # Panics
    /// If `needle` does not appear before the timeout.
    pub fn read_until(&mut self, needle: &str) -> String {
        let deadline = Instant::now() + self.timeout;
        loop {
            if let Some(at) = self.seen[self.cursor..].find(needle) {
                let start = self.cursor;
                let end = self.cursor + at;
                self.cursor = end + needle.len();
                return self.seen[start..end].to_string();
            }
            assert!(
                Instant::now() < deadline,
                "timed out after {:?} waiting for {needle:?}\n{}",
                self.timeout,
                self.render_context()
            );
            // Read once, then re-check: output can arrive in the same wakeup
            // that closes the pty, so EOF alone does not mean the needle is
            // absent.
            let still_open = self.pump(deadline);
            assert!(
                still_open || self.seen[self.cursor..].contains(needle),
                "shell exited before producing {needle:?}\n{}",
                self.render_context()
            );
        }
    }

    /// Read up to and including the next prompt, returning the output before it
    /// with the trailing newline stripped. This is the workhorse: it is exactly
    /// "what did the shell print in response to my command".
    pub fn read_until_prompt(&mut self) -> String {
        let raw = self.read_until(PROMPT);
        normalize(&raw)
    }

    /// Assert the shell prints exactly `expected` before the next prompt.
    ///
    /// `expected` is matched byte-for-byte after CR-LF normalization (a PTY
    /// echoes `\r\n` for `\n`; the tester compares the logical lines).
    pub fn expect_output(&mut self, expected: &str) {
        let actual = self.read_until_prompt();
        assert_eq!(
            actual,
            expected,
            "\n  expected: {expected:?}\n  actual:   {actual:?}\n{}",
            self.render_context()
        );
    }

    /// Send a command and assert its full output, up to the following prompt.
    ///
    /// Echoed input is stripped: a PTY echoes the typed line back, and every
    /// stage's expected output is what the shell *printed*, not what was typed.
    pub fn expect_command(&mut self, command: &str, expected: &str) {
        self.send_line(command);
        let echoed = self.read_until("\n");
        assert_eq!(
            normalize(&echoed),
            command,
            "expected the pty to echo the command back\n{}",
            self.render_context()
        );
        self.expect_output(expected);
    }

    /// Wait for the initial prompt the shell prints before reading any input.
    pub fn expect_prompt(&mut self) {
        self.read_until(PROMPT);
    }

    /// Assert the shell has exited, and return its exit status.
    ///
    /// # Panics
    /// If the shell is still running after the timeout.
    pub fn expect_exit(&mut self) -> portable_pty::ExitStatus {
        let deadline = Instant::now() + self.timeout;
        loop {
            if let Some(status) = self.child.try_wait().expect("failed to poll shell") {
                return status;
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                panic!(
                    "shell still running {:?} after being asked to exit\n{}",
                    self.timeout,
                    self.render_context()
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Assert the shell is still running, i.e. it did not die on the last input.
    pub fn expect_alive(&mut self) {
        let exited = self.child.try_wait().expect("failed to poll shell");
        assert!(
            exited.is_none(),
            "shell exited unexpectedly with {exited:?}\n{}",
            self.render_context()
        );
    }

    /// Everything the shell has printed so far.
    pub fn transcript(&self) -> &str {
        &self.seen
    }

    fn render_context(&self) -> String {
        format!(
            "  --- transcript so far ---\n{}\n  --- end transcript ---",
            if self.seen.is_empty() {
                "  (nothing)".to_string()
            } else {
                self.seen
                    .lines()
                    .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Never leave a shell behind; a leaked process would hold the sandbox
        // tempdir open and could outlive the suite.
        let _ = self.child.kill();
        let _ = self.child.wait();
        drop(self.master.take_writer());
    }
}

/// Collapse the PTY's `\r\n` into `\n` and drop one trailing line ending.
///
/// A terminal in canonical mode translates the shell's `\n` on output to
/// `\r\n`. That is a property of the tty, not of the shell, so it must not
/// leak into assertions. The trailing `\r` is stripped separately because a
/// caller that read up to a `\n` delimiter has already consumed the `\n` and
/// left the `\r` behind.
fn normalize(raw: &str) -> String {
    let s = raw.replace("\r\n", "\n");
    let s = s.strip_suffix('\n').unwrap_or(&s);
    s.strip_suffix('\r').unwrap_or(s).to_string()
}
