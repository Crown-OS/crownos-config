//! The key of a running program, from what its process shows: the
//! executable, the command line and the working directory. The caller reads
//! those once, when the program connects.

use std::path::{Path, PathBuf};

use super::AppKey;

/// Interpreters and runtimes whose programs are named by the script, jar or
/// app they run, by file name prefix. A version suffix such as `3.13` is
/// allowed.
const RUNTIMES: [&str; 14] = [
    "python", "node", "nodejs", "bun", "deno", "perl", "ruby", "bash", "sh", "dash", "zsh", "lua",
    "java", "electron",
];
/// Words between a runtime and its program that are not the program.
const SUBCOMMANDS: [&str; 1] = ["run"];
/// Options of a runtime that take the next argument as their value.
const OPTIONS_WITH_VALUE: [&str; 6] = ["-cp", "-classpath", "--class-path", "-m", "-c", "-e"];

impl AppKey {
    /// The key of a program that runs `executable` with `arguments` (the
    /// program name first) in `working_directory`.
    ///
    /// A runtime is keyed with the program it runs: the script, `-jar` jar
    /// or Electron app. Then one grant does not cover every app on the
    /// runtime. A runtime without such a program on its command line, such
    /// as `python -c` or `java Main`, has no key.
    pub fn of_program(
        executable: &Path,
        arguments: &[String],
        working_directory: Option<&Path>,
    ) -> Option<Self> {
        if !is_runtime(executable) {
            return Some(Self::HostExecutable {
                path: executable.to_path_buf(),
            });
        }
        let program = program_argument(arguments)?;
        let program = PathBuf::from(program);
        let script = if program.is_absolute() {
            program
        } else {
            working_directory?.join(program)
        };
        Some(Self::Interpreted {
            interpreter: executable.to_path_buf(),
            script,
        })
    }
}

fn is_runtime(executable: &Path) -> bool {
    executable
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            RUNTIMES.iter().any(|runtime| {
                name.strip_prefix(runtime).is_some_and(|version| {
                    version
                        .chars()
                        .all(|character| character.is_ascii_digit() || character == '.')
                })
            })
        })
}

/// The script, jar or app on a runtime's command line. `-jar` names it
/// explicitly; otherwise it is the first argument that is not an option.
/// `-c`, `-e` and `-m` run code without a file, so there is none.
fn program_argument(arguments: &[String]) -> Option<&str> {
    let mut rest = arguments.iter().skip(1).map(String::as_str);
    while let Some(argument) = rest.next() {
        match argument {
            "-jar" => return rest.next(),
            "-c" | "-e" | "-m" => return None,
            option if OPTIONS_WITH_VALUE.contains(&option) => {
                rest.next();
            }
            option if option.starts_with('-') || SUBCOMMANDS.contains(&option) => {}
            program => return Some(program),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(executable: &str, arguments: &[&str], working_directory: &str) -> Option<AppKey> {
        let arguments: Vec<String> = arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
        AppKey::of_program(
            Path::new(executable),
            &arguments,
            Some(Path::new(working_directory)),
        )
    }

    fn interpreted(interpreter: &str, script: &str) -> Option<AppKey> {
        Some(AppKey::Interpreted {
            interpreter: interpreter.into(),
            script: script.into(),
        })
    }

    #[test]
    fn a_native_program_is_its_executable() {
        assert_eq!(
            key("/usr/bin/crown-contextd", &["crown-contextd"], "/"),
            Some(AppKey::HostExecutable {
                path: "/usr/bin/crown-contextd".into()
            })
        );
        assert_eq!(
            key("/usr/bin/shotwell", &["shotwell"], "/"),
            Some(AppKey::HostExecutable {
                path: "/usr/bin/shotwell".into()
            })
        );
    }

    #[test]
    fn runtimes_are_keyed_with_the_program_they_run() {
        assert_eq!(
            key("/usr/bin/python3.13", &["python3", "-u", "bot.py"], "/srv"),
            interpreted("/usr/bin/python3.13", "/srv/bot.py")
        );
        assert_eq!(
            key("/usr/bin/node", &["node", "/opt/app/main.js"], "/"),
            interpreted("/usr/bin/node", "/opt/app/main.js")
        );
        assert_eq!(
            key(
                "/usr/lib/jvm/bin/java",
                &["java", "-Xmx1g", "-jar", "/opt/ide.jar"],
                "/"
            ),
            interpreted("/usr/lib/jvm/bin/java", "/opt/ide.jar")
        );
        assert_eq!(
            key(
                "/usr/lib/electron33/electron",
                &["electron", "--no-sandbox", "/usr/lib/app/app.asar"],
                "/"
            ),
            interpreted("/usr/lib/electron33/electron", "/usr/lib/app/app.asar")
        );
        assert_eq!(
            key("/usr/bin/deno", &["deno", "run", "agent.ts"], "/home/u"),
            interpreted("/usr/bin/deno", "/home/u/agent.ts")
        );
    }

    #[test]
    fn a_runtime_without_a_program_file_has_no_key() {
        assert_eq!(
            key("/usr/bin/python3", &["python3", "-c", "print(1)"], "/"),
            None
        );
        assert_eq!(
            key("/usr/bin/python3", &["python3", "-m", "http.server"], "/"),
            None
        );
        assert_eq!(
            key("/usr/bin/java", &["java", "-cp", "lib", "Main"], "/"),
            interpreted("/usr/bin/java", "/Main")
        );
        assert_eq!(key("/usr/bin/bash", &["bash"], "/"), None);
    }
}
