//! The key of a running program, from what its process shows: the
//! executable, the command line and the working directory. The caller reads
//! those once, when the program connects.

use std::path::{Path, PathBuf};

use super::AppKey;

mod flatpak;
mod launchers;
mod runtimes;

pub use flatpak::{FLATPAK_ENGINE, USER_FLATPAK_ENGINE};
pub use launchers::{LAUNCHER_FILE, LauncherFileError, LauncherMap};

use runtimes::{ELECTRON, KEYED_RUNTIMES, Runtime, UNKEYED_RUNTIMES};

impl AppKey {
    /// The key of a program that runs `executable` with `arguments` (the
    /// program name first) in `working_directory`.
    ///
    /// A known runtime is keyed with the program it runs: the script, the
    /// `-jar` jar or the Electron app. Then one grant does not cover every
    /// app on the runtime. A runtime gets no key when its command line runs
    /// code without a file (`python -c`, `java -m`), names a Java class
    /// instead of a jar (`java Main`), or has an option this code does not
    /// know, which could hide the real program. Loaders and some
    /// interpreters (`ld.so`, `busybox`, `php` …) never get a key.
    pub fn of_program(
        executable: &Path,
        arguments: &[String],
        working_directory: Option<&Path>,
    ) -> Option<Self> {
        let name = executable.file_name()?.to_str()?;
        if UNKEYED_RUNTIMES
            .iter()
            .any(|unkeyed| name.starts_with(unkeyed))
        {
            return None;
        }
        let Some(runtime) = KEYED_RUNTIMES
            .iter()
            .chain([&ELECTRON])
            .find(|runtime| is_named(runtime, name))
        else {
            return Some(Self::HostExecutable {
                path: executable.to_path_buf(),
            });
        };
        let program = PathBuf::from(program_argument(runtime, arguments)?);
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

fn is_named(runtime: &Runtime, file_name: &str) -> bool {
    runtime.names.iter().any(|name| {
        file_name.strip_prefix(name).is_some_and(|version| {
            version
                .chars()
                .all(|character| character.is_ascii_digit() || character == '.')
        })
    })
}

/// The program on a runtime's command line, or `None` when there is none
/// or the command line holds an option this code does not know.
fn program_argument<'a>(runtime: &Runtime, arguments: &'a [String]) -> Option<&'a str> {
    let mut rest = arguments.iter().skip(1).map(String::as_str);
    while let Some(argument) = rest.next() {
        if runtime.program_option == Some(argument) {
            return rest.next();
        }
        if runtime.inline_code.contains(&argument)
            || runtime.inline_code.iter().any(|option| {
                option.len() == 2 && argument.starts_with(option) && argument.len() > 2
            })
        {
            return None;
        }
        if runtime.with_value.contains(&argument) {
            rest.next()?;
            continue;
        }
        let known = runtime.flags.contains(&argument)
            || runtime.subcommands.contains(&argument)
            || runtime
                .attached
                .iter()
                .any(|option| argument.starts_with(option) && argument.len() > option.len())
            || (runtime.long_options_with_equals
                && argument.starts_with("--")
                && argument.contains('='));
        if known {
            continue;
        }
        if argument.starts_with('-') {
            return None;
        }
        return runtime.program_option.is_none().then_some(argument);
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
            key("/usr/bin/shotwell", &["shotwell", "-x"], "/"),
            Some(AppKey::HostExecutable {
                path: "/usr/bin/shotwell".into()
            })
        );
    }

    #[test]
    fn runtimes_are_keyed_with_the_program_they_run() {
        assert_eq!(
            key(
                "/usr/bin/python3.13",
                &["python3", "-u", "-W", "ignore", "bot.py"],
                "/srv"
            ),
            interpreted("/usr/bin/python3.13", "/srv/bot.py")
        );
        assert_eq!(
            key(
                "/usr/bin/node",
                &[
                    "node",
                    "--max-old-space-size=512",
                    "-r",
                    "dotenv/config",
                    "/opt/app/main.js"
                ],
                "/"
            ),
            interpreted("/usr/bin/node", "/opt/app/main.js")
        );
        assert_eq!(
            key(
                "/usr/lib/jvm/bin/java",
                &["java", "-Xmx1g", "-Dkey=value", "-jar", "/opt/ide.jar"],
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
            key(
                "/usr/bin/deno",
                &["deno", "run", "-A", "agent.ts"],
                "/home/u"
            ),
            interpreted("/usr/bin/deno", "/home/u/agent.ts")
        );
    }

    #[test]
    fn option_values_are_not_taken_for_the_program() {
        assert_eq!(
            key(
                "/usr/bin/python3",
                &["python3", "-W", "/srv/decoy.py", "/srv/real.py"],
                "/"
            ),
            interpreted("/usr/bin/python3", "/srv/real.py")
        );
        assert_eq!(
            key(
                "/usr/bin/node",
                &["node", "--require", "/srv/decoy.js", "/srv/real.js"],
                "/"
            ),
            interpreted("/usr/bin/node", "/srv/real.js")
        );
        assert_eq!(
            key(
                "/usr/bin/java",
                &["java", "-cp", "/srv/decoy.jar", "-jar", "/srv/real.jar"],
                "/"
            ),
            interpreted("/usr/bin/java", "/srv/real.jar")
        );
    }

    #[test]
    fn a_command_line_that_could_hide_the_program_has_no_key() {
        for arguments in [
            &["python3", "-c", "print(1)"][..],
            &["python3", "-m", "http.server"],
            &["python3", "-cprint(1)", "/srv/decoy.py"],
            &["python3", "--unknown-option", "/srv/decoy.py"],
            &["node", "-e", "1", "/srv/decoy.js"],
            &["node", "--inspect", "/srv/decoy.js"],
            &["bash", "-c", "true", "/srv/decoy.sh"],
            &["bash"],
            &["electron", "--unknown", "/srv/app"],
        ] {
            let runtime = match arguments[0] {
                "python3" => "/usr/bin/python3",
                "node" => "/usr/bin/node",
                "bash" => "/usr/bin/bash",
                _ => "/usr/bin/electron",
            };
            assert_eq!(key(runtime, arguments, "/"), None, "{arguments:?}");
        }
    }

    #[test]
    fn a_java_class_is_not_a_program_file() {
        assert_eq!(key("/usr/bin/java", &["java", "Main"], "/srv"), None);
        assert_eq!(
            key("/usr/bin/java", &["java", "-cp", "lib", "Main"], "/srv"),
            None
        );
    }

    #[test]
    fn loaders_and_unlisted_interpreters_get_no_key() {
        for executable in [
            "/usr/lib/ld-linux-x86-64.so.2",
            "/usr/lib/ld.so",
            "/usr/bin/busybox",
            "/usr/bin/php",
            "/usr/bin/pypy3",
            "/usr/bin/gjs",
        ] {
            assert_eq!(
                key(executable, &["x", "/srv/app"], "/"),
                None,
                "{executable}"
            );
        }
    }
}
