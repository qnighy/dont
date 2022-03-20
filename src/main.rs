// Copyright 2022 Masaki Hara
// See LICENSE.txt and LICENSE-Apache-2.0.txt for the license.

use std::ffi::OsString;
use std::io;

use cfg_if::cfg_if;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(allow_hyphen_values = true)]
    command: Vec<OsString>,
}

fn main() {
    let args = Args::parse();
    match execute(&DefaultController, &args) {
        Conclusion::Exit(code) => {
            std::process::exit(code);
        }
        Conclusion::Exec(args) => {
            use std::process::Command;
            let mut command = Command::new(&args[0]);
            command.args(&args[1..]);
            let result: Result<(), io::Error>;
            cfg_if! {
                if #[cfg(unix)] {
                    use std::os::unix::process::CommandExt;
                    result = Err(command.exec());
                } else {
                    result = command.spawn();
                }
            }
            if let Err(e) = result {
                let command_description = args
                    .into_iter()
                    .map(|x| x.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ");
                eprintln!("Failed to run {}: {}", command_description, e);
                std::process::exit(1);
            }
        }
    }
}

fn execute<C: Controller>(ctl: &C, args: &Args) -> Conclusion {
    if args.command.len() == 0 {
        // Just "dont". What is the right reaction to the command?
        return Conclusion::Exit(0);
    }
    if args.command[0] == "true" {
        return Conclusion::Exit(1);
    } else if args.command[0] == "false" {
        return Conclusion::Exit(0);
    } else if args.command[0] == "dont" {
        if args.command.len() == 1 {
            // Just "dont dont". What is the right reaction to the command?
            return Conclusion::Exit(0);
        }
        return Conclusion::Exec(args.command[1..].to_owned());
    } else if args.command[0] == "ls" && ctl.has_command("sl") {
        return Conclusion::Exec(
            vec![OsString::from("sl")]
                .into_iter()
                .chain(args.command[1..].iter().cloned())
                .collect(),
        );
    } else if args.command[0] == "sl" {
        return Conclusion::Exec(
            vec![OsString::from("ls")]
                .into_iter()
                .chain(args.command[1..].iter().cloned())
                .collect(),
        );
    } else if args.command[0] == "vim" && ctl.has_command("emacs") {
        return Conclusion::Exec(
            vec![OsString::from("emacs")]
                .into_iter()
                .chain(args.command[1..].iter().cloned())
                .collect(),
        );
    } else if args.command[0] == "emacs" && ctl.has_command("vim") {
        return Conclusion::Exec(
            vec![OsString::from("vim")]
                .into_iter()
                .chain(args.command[1..].iter().cloned())
                .collect(),
        );
    }
    Conclusion::Exit(0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Conclusion {
    Exit(i32),
    Exec(Vec<OsString>),
}

#[cfg_attr(test, mockall::automock)]
trait Controller {
    fn has_command(&self, name: &str) -> bool;
}

#[derive(Debug)]
struct DefaultController;

impl Controller for DefaultController {
    fn has_command(&self, name: &str) -> bool {
        which::which(name).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    fn main(ctl: &MockController, args: &[&str]) -> Result<Conclusion, clap::Error> {
        // TODO: resolve unwrap correctly
        let args = Args::try_parse_from(args)?;
        Ok(execute(ctl, &args))
    }

    #[test]
    fn test_help() {
        let ctl = MockController::new();
        let e = main(&ctl, &["dont", "--help"]).unwrap_err();
        let msg = e.to_string();
        assert!(
            msg.contains("USAGE:"),
            "Expected message to contain \"USAGE:\", got {}",
            msg
        );
    }

    #[test]
    fn test_true() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "true"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(1));
    }

    #[test]
    fn test_true_with_dashes() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "--", "true"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(1));
    }

    #[test]
    fn test_false() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "false"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(0));
    }

    #[test]
    fn test_dont() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "dont", "ls"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["ls".into()]));
    }

    #[test]
    fn test_dont_with_dashes() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "--", "dont", "ls"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["ls".into()]));
    }

    #[test]
    fn test_dont_with_wrong_dashes() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "dont", "--", "ls"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["--".into(), "ls".into()]));
    }

    #[test]
    fn test_ls() {
        let mut ctl = MockController::new();
        ctl.expect_has_command().with(eq("sl")).returning(|_| false);
        let concl = main(&ctl, &["dont", "ls"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(0));
    }

    #[test]
    fn test_ls_when_sl_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command().with(eq("sl")).returning(|_| true);
        let concl = main(&ctl, &["dont", "ls"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["sl".into()]));
    }

    #[test]
    fn test_ls_with_args_when_sl_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command().with(eq("sl")).returning(|_| true);
        let concl = main(&ctl, &["dont", "ls", "foo"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["sl".into(), "foo".into()]));
    }

    #[test]
    fn test_sl() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "sl"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["ls".into()]));
    }

    #[test]
    fn test_sl_with_args() {
        let ctl = MockController::new();
        let concl = main(&ctl, &["dont", "sl", "foo"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["ls".into(), "foo".into()]));
    }

    #[test]
    fn test_vim() {
        let mut ctl = MockController::new();
        ctl.expect_has_command()
            .with(eq("emacs"))
            .returning(|_| false);
        let concl = main(&ctl, &["dont", "vim"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(0));
    }

    #[test]
    fn test_vim_when_emacs_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command()
            .with(eq("emacs"))
            .returning(|_| true);
        let concl = main(&ctl, &["dont", "vim"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["emacs".into()]));
    }

    #[test]
    fn test_vim_with_args_when_emacs_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command()
            .with(eq("emacs"))
            .returning(|_| true);
        let concl = main(&ctl, &["dont", "vim", "foo"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["emacs".into(), "foo".into()]));
    }

    #[test]
    fn test_emacs() {
        let mut ctl = MockController::new();
        ctl.expect_has_command()
            .with(eq("vim"))
            .returning(|_| false);
        let concl = main(&ctl, &["dont", "emacs"]).unwrap();
        assert_eq!(concl, Conclusion::Exit(0));
    }

    #[test]
    fn test_emacs_when_vim_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command().with(eq("vim")).returning(|_| true);
        let concl = main(&ctl, &["dont", "emacs"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["vim".into()]));
    }

    #[test]
    fn test_emacs_with_args_when_vim_exists() {
        let mut ctl = MockController::new();
        ctl.expect_has_command().with(eq("vim")).returning(|_| true);
        let concl = main(&ctl, &["dont", "emacs", "foo"]).unwrap();
        assert_eq!(concl, Conclusion::Exec(vec!["vim".into(), "foo".into()]));
    }
}
