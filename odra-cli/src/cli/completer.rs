//! Tab-completion for the interactive REPL.
//!
//! Walks the assembled clap command tree — the same one the REPL parses against — to suggest
//! subcommand names, nested subcommands (e.g. `inspect <Contract>`) and `--flags` for the command
//! under the cursor.
//!
//! This is unrelated to the `completions` subcommand: that one emits a static script for an
//! external shell (bash/zsh/...) and has no effect inside the REPL's own line editor. Here the
//! editor does the completing in-process, so it works regardless of how the binary was launched.

use clap::Command;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

/// REPL line-editor helper. Only completion is customized; hinting, highlighting and validation
/// keep rustyline's defaults.
pub(super) struct ReplHelper {
    /// The command tree to complete against — built once, with `repl` already excluded.
    command: Command,
    /// First-word commands handled by the REPL loop itself rather than clap (`help`/`exit`/...).
    builtins: Vec<String>
}

impl ReplHelper {
    pub(super) fn new(command: Command, builtins: &[&str]) -> Self {
        Self {
            command,
            builtins: builtins.iter().map(|s| s.to_string()).collect()
        }
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        Ok(complete(&self.command, &self.builtins, line, pos))
    }
}

// Hinting / highlighting / validation are left at their defaults.
impl Hinter for ReplHelper {
    type Hint = String;
}
impl Highlighter for ReplHelper {}
impl Validator for ReplHelper {}
impl Helper for ReplHelper {}

/// Computes completion candidates for `line` at byte offset `pos`.
///
/// Returns the start offset of the word being completed and the candidates that replace it. Kept as
/// a free function (no rustyline `Context`) so it can be unit-tested directly.
fn complete(root: &Command, builtins: &[String], line: &str, pos: usize) -> (usize, Vec<Pair>) {
    let head = &line[..pos.min(line.len())];

    // The word under the cursor. A trailing space means we're starting a fresh word.
    let word_start = head.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
    let current = &head[word_start..];

    // Tokens already completed before the current word. Whitespace splitting is enough for
    // command/flag tokens; quoted argument values are never completion targets here.
    let prior: Vec<&str> = head[..word_start].split_whitespace().collect();

    // Descend the command tree following the recognized subcommand tokens.
    let mut cmd = root;
    let mut descended = false;
    for token in &prior {
        if token.starts_with('-') {
            continue; // a flag, not a subcommand
        }
        match cmd.find_subcommand(token) {
            Some(sub) => {
                cmd = sub;
                descended = true;
            }
            // An argument value (e.g. a contract address) — we can't predict the next word.
            None => return (word_start, Vec::new())
        }
    }

    let mut candidates: Vec<String> = Vec::new();
    if current.starts_with('-') {
        // Complete the long flags of the current command.
        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                candidates.push(format!("--{long}"));
            }
        }
    } else {
        // Complete (nested) subcommand names...
        for sub in cmd.get_subcommands() {
            candidates.push(sub.get_name().to_string());
        }
        // ...plus the REPL built-ins, but only as a first word.
        if !descended {
            candidates.extend(builtins.iter().cloned());
        }
    }

    let mut pairs: Vec<Pair> = candidates
        .into_iter()
        .filter(|c| c.starts_with(current))
        .map(|c| Pair {
            display: c.clone(),
            replacement: c
        })
        .collect();
    pairs.sort_by(|a, b| a.display.cmp(&b.display));
    pairs.dedup_by(|a, b| a.display == b.display);

    (word_start, pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    fn sample() -> Command {
        Command::new("Odra CLI")
            .subcommand(Command::new("deploy").arg(Arg::new("gas").long("gas")))
            .subcommand(
                Command::new("inspect")
                    .subcommand(Command::new("Token"))
                    .subcommand(Command::new("Auction"))
            )
            .subcommand(Command::new("status"))
    }

    fn names(pairs: &[Pair]) -> Vec<String> {
        pairs.iter().map(|p| p.replacement.clone()).collect()
    }

    fn builtins() -> Vec<String> {
        vec!["help".to_string(), "exit".to_string(), "quit".to_string()]
    }

    #[test]
    fn completes_top_level_subcommands_by_prefix() {
        let (start, pairs) = complete(&sample(), &builtins(), "in", 2);
        assert_eq!(start, 0);
        assert_eq!(names(&pairs), vec!["inspect"]);
    }

    #[test]
    fn includes_builtins_only_as_first_word() {
        let cmd = sample();

        let (_, pairs) = complete(&cmd, &builtins(), "", 0);
        let n = names(&pairs);
        assert!(n.contains(&"help".to_string()));
        assert!(n.contains(&"deploy".to_string()));

        // After a subcommand the built-ins must not be offered.
        let (_, pairs) = complete(&cmd, &builtins(), "inspect ", 8);
        let n = names(&pairs);
        assert!(!n.contains(&"help".to_string()));
        assert!(n.contains(&"Token".to_string()));
        assert!(n.contains(&"Auction".to_string()));
    }

    #[test]
    fn completes_nested_subcommands_by_prefix() {
        let (start, pairs) = complete(&sample(), &builtins(), "inspect To", 10);
        assert_eq!(start, 8);
        assert_eq!(names(&pairs), vec!["Token"]);
    }

    #[test]
    fn completes_flags_of_current_command() {
        let (_start, pairs) = complete(&sample(), &builtins(), "deploy --g", 10);
        assert_eq!(names(&pairs), vec!["--gas"]);
    }

    #[test]
    fn unknown_argument_value_yields_no_candidates() {
        // `status` has no subcommands; a stray token means the next word is unpredictable.
        let (start, pairs) = complete(&sample(), &builtins(), "status foo ", 11);
        assert_eq!(start, 11);
        assert!(pairs.is_empty());
    }
}
