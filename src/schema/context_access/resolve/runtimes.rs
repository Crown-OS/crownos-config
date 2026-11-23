//! How each known runtime spells its command line: which options take a
//! value, which run code without a file, and which name the program.

/// One runtime, known by the file name of its executable.
#[derive(Debug)]
pub(super) struct Runtime {
    /// File name prefixes; a version suffix such as `3.13` may follow.
    pub names: &'static [&'static str],
    /// Options without a value.
    pub flags: &'static [&'static str],
    /// Options whose value is the next argument.
    pub with_value: &'static [&'static str],
    /// Options that may carry their value attached, such as `-Wdefault`.
    pub attached: &'static [&'static str],
    /// Options that run code from the command line or a module, not a file.
    pub inline_code: &'static [&'static str],
    /// The option whose value is the program, such as java's `-jar`.
    pub program_option: Option<&'static str>,
    /// Words between the runtime and its program, such as `run`.
    pub subcommands: &'static [&'static str],
    /// `--name=value` options are allowed.
    pub long_options_with_equals: bool,
}

const fn runtime(names: &'static [&'static str]) -> Runtime {
    Runtime {
        names,
        flags: &[],
        with_value: &[],
        attached: &[],
        inline_code: &[],
        program_option: None,
        subcommands: &[],
        long_options_with_equals: false,
    }
}

pub(super) const KEYED_RUNTIMES: [Runtime; 9] = [
    Runtime {
        flags: &[
            "-u", "-B", "-E", "-I", "-O", "-OO", "-s", "-S", "-q", "-b", "-bb", "-d", "-P", "-R",
            "-x",
        ],
        with_value: &["-W", "-X"],
        attached: &["-W", "-X"],
        inline_code: &["-c", "-m"],
        ..runtime(&["python"])
    },
    Runtime {
        flags: &[
            "--enable-source-maps",
            "--no-warnings",
            "--no-deprecation",
            "--trace-warnings",
            "--trace-uncaught",
            "--preserve-symlinks",
        ],
        with_value: &["-r", "--require", "--import", "--loader"],
        inline_code: &["-e", "--eval", "-p", "--print"],
        long_options_with_equals: true,
        ..runtime(&["node", "nodejs"])
    },
    Runtime {
        flags: &["-A", "--no-check", "--quiet", "-q"],
        subcommands: &["run"],
        inline_code: &["eval"],
        long_options_with_equals: true,
        ..runtime(&["deno"])
    },
    Runtime {
        flags: &["--bun", "--smol", "--hot", "--watch"],
        subcommands: &["run"],
        inline_code: &["-e", "--eval", "-p", "--print"],
        ..runtime(&["bun"])
    },
    Runtime {
        flags: &["-w", "-W", "-X", "-T", "-t", "-s", "-n", "-p", "-a", "-l"],
        with_value: &["-I", "-M", "-m"],
        attached: &["-I", "-M", "-m"],
        inline_code: &["-e", "-E"],
        ..runtime(&["perl"])
    },
    Runtime {
        flags: &["-w", "-W", "-v", "--disable-gems"],
        with_value: &["-I", "-r"],
        attached: &["-I", "-r"],
        inline_code: &["-e"],
        ..runtime(&["ruby"])
    },
    Runtime {
        flags: &[
            "-e",
            "-u",
            "-x",
            "-v",
            "-n",
            "-l",
            "-i",
            "-r",
            "--login",
            "--norc",
            "--noprofile",
            "--posix",
        ],
        with_value: &["-o", "+o"],
        inline_code: &["-c", "-s"],
        ..runtime(&["bash", "sh", "dash", "zsh"])
    },
    Runtime {
        with_value: &["-l"],
        attached: &["-l"],
        flags: &["-i", "-v", "-E", "-W"],
        inline_code: &["-e"],
        ..runtime(&["lua"])
    },
    Runtime {
        flags: &[
            "-ea", "-da", "-esa", "-dsa", "-server", "-client", "-verbose",
        ],
        with_value: &["-cp", "-classpath", "--class-path", "-p", "--module-path"],
        attached: &["-D", "-X", "-javaagent:", "-agentlib:", "-agentpath:"],
        inline_code: &["-m", "--module"],
        program_option: Some("-jar"),
        ..runtime(&["java"])
    },
];

/// Electron apps: any flag is the app's own business, so only the first
/// argument that is not an option names the app.
pub(super) const ELECTRON: Runtime = Runtime {
    long_options_with_equals: true,
    flags: &[
        "--no-sandbox",
        "--disable-gpu",
        "--disable-gpu-sandbox",
        "--enable-wayland-ime",
        "--ozone-platform-hint=auto",
    ],
    ..runtime(&["electron"])
};

/// Interpreters and loaders that run any program in ways their command line
/// does not reliably show. Their clients get no key.
pub(super) const UNKEYED_RUNTIMES: [&str; 7] = [
    "php", "pypy", "gjs", "busybox", "ld.so", "ld-linux", "ld-musl",
];
