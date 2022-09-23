use argh::FromArgs;
use std::{
    env,
    fs::{self, File},
    io::{Error, ErrorKind, Read as _, Write as _},
    path::Path,
    process,
    str::FromStr,
};

const BASE: &str = "timeit";
const BASE_DIR: &str = "rust-timeit";
const CARGO_TOML: &str = include_str!("Cargo.toml.tmpl");
const TIMEIT_EXPRESSION: &str = include_str!("expression.rs");
const TIMEIT_RS: &str = include_str!("timeit.rs");

const CYCLES_DEP: &str = r#"criterion-cycles-per-byte = "0.1.2""#;
const PERF_DEP: &str = r#"criterion-linux-perf = "0.1""#;
const CYCLES_USE: &str = "criterion_cycles_per_byte::CyclesPerByte";
const PERF_USE: &str = "criterion_linux_perf::{PerfMeasurement, PerfMode}";

macro_rules! perf_mode {
    ( $( $ident:ident => $word:literal, )* ) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        enum PerfMode {
            $( $ident, )*
        }

        impl PerfMode {
            fn as_perf_mode(&self) -> &'static str {
                match self {
                    $( Self::$ident => stringify!($ident), )*
                }
            }

            fn all_modes() -> Vec<&'static str> {
                vec![ $( $word, )* ]
            }
        }

        impl FromStr for PerfMode {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, String> {
                match s {
                    "help" => {
                        eprintln!("Valid values for --perf");
                        for mode in Self::all_modes() {
                            eprintln!("  {}", mode);
                        }
                        process::exit(1);
                    }
                    $( $word => Ok(Self::$ident), )*
                    _ => Err("Unknown perf mode".into()),
                }
            }
        }
    };
}

perf_mode! {
    Cycles => "cycles",
    Instructions => "instructions",
    Branches => "branches",
    BranchMisses => "branch-misses",
    CacheRefs => "cache-refs",
    CacheMisses => "cache-misses",
    BusCycles => "bus-cycles",
    RefCycles => "ref-cycles",
}

#[derive(Debug, FromArgs)]
#[argh(description = r#"Tool for measuring execution time of small Rust code snippets."#)]
struct Args {
    /// code to be executed once before timing begins
    #[argh(option, short = 's')]
    setup: Option<String>,

    /// crate name and version to add to the dependencies section
    #[argh(option, short = 'd')]
    dependency: Vec<String>,

    /// add an extra "use" line
    #[argh(option, short = 'u', long = "use")]
    uses: Vec<String>,

    /// include the named file's contents in the source code
    #[argh(option, short = 'i')]
    include: Vec<String>,

    /// use the CPU cycle count instead of wall time
    #[argh(switch)]
    cycles: bool,

    /// use an alternate measurement instead of wall time (use `--perf
    /// help` to list all the options for this)
    #[cfg(target_os = "linux")]
    #[argh(option, short = 'p')]
    perf: Option<PerfMode>,

    /// wrap the expressions in `criterion::black_box` to ensure their full evaluation
    #[argh(switch, short = 'b')]
    black_box: bool,

    /// delete the cache directory before starting, making a fresh start
    #[argh(switch, short = 'f')]
    fresh: bool,

    /// clean up the cache directory after a successful finish
    #[argh(switch, short = 'c')]
    cleanup: bool,

    /// enable verbose mode
    #[argh(switch, short = 'v')]
    verbose: bool,

    #[argh(positional)]
    expression: Vec<String>,
}

impl Args {
    fn dependencies(&mut self) -> String {
        if self.cycles {
            self.dependency.push(CYCLES_DEP.into());
        }
        #[cfg(target_os = "linux")]
        if self.perf.is_some() {
            self.dependency.push(PERF_DEP.into());
        }
        self.dependency.join("\n")
    }

    fn uses(&mut self) -> String {
        if self.cycles {
            self.uses.push(CYCLES_USE.into());
        }
        #[cfg(target_os = "linux")]
        if self.perf.is_some() {
            self.uses.push(PERF_USE.into());
        }
        self.uses
            .iter()
            .map(|import| format!("use {};\n", import))
            .collect::<Vec<_>>()
            .join("")
    }

    fn includes(&self) -> Result<String, Error> {
        self.include
            .iter()
            .map(|filename| {
                let mut contents = String::new();
                fs::File::open(filename)
                    .and_then(|mut file| file.read_to_string(&mut contents))
                    .map(move |_| contents)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|includes| includes.join("\n"))
    }

    fn setup(&self) -> String {
        self.setup
            .as_ref()
            .map(|s| format!("{};", s))
            .unwrap_or_default()
    }

    fn expressions(&self) -> String {
        self.expression
            .iter()
            .map(|expression| {
                let black_box = if self.black_box { "black_box" } else { "" };
                TIMEIT_EXPRESSION
                    .replace("/*BLACK_BOX*/", black_box)
                    .replace("/*EXPRESSION*/", expression)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn timer(&self) -> String {
        #[cfg(target_os = "linux")]
        if let Some(mode) = self.perf {
            return format!("PerfMeasurement::new(PerfMode::{})", mode.as_perf_mode());
        }
        if self.cycles {
            "CyclesPerByte".into()
        } else {
            "WallTime".into()
        }
    }
}

fn create(filename: &str, template: &str, subst: &[(&str, &str)]) -> Result<(), Error> {
    let tempname = format!("{}.tmp", filename);
    let mut data = template.to_string();
    for (key, value) in subst {
        data = data.replace(key, value);
    }

    let mut out = File::create(&tempname)?;
    out.write_all(data.as_bytes())?;
    out.flush()?;
    drop(out);

    fs::rename(tempname, filename)
}

fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    fs::remove_dir_all(path).or_else(|error| match error.kind() {
        ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })
}

fn main() -> Result<(), Error> {
    let mut args = argh::from_env::<Args>();
    if args.expression.is_empty() {
        eprintln!("Please specify at least one expression");
        process::exit(1);
    }

    #[cfg(target_os = "linux")]
    if args.cycles && args.perf.is_some() {
        eprintln!("Cannot specify both --cycles and --perf");
        process::exit(1);
    }

    // Pre-load the included files before changing the working directory
    let includes = args.includes()?;

    let mut base_dir = dirs::cache_dir().expect("Could not determine cache directory");
    base_dir.push(BASE_DIR);
    if args.verbose {
        println!("Using cache directory {:?}.", base_dir);
    }
    if args.fresh {
        println!("Deleting cache directory.");
        remove_dir_all(&base_dir)?;
    }
    fs::create_dir_all(&base_dir)?;
    env::set_current_dir(&base_dir)?;
    fs::create_dir_all("benches")?;

    create(
        "Cargo.toml",
        CARGO_TOML,
        &[("@DEPENDENCIES@", &args.dependencies()), ("@BASE@", BASE)],
    )?;

    create(
        &format!("benches/{}.rs", BASE),
        TIMEIT_RS,
        &[
            ("/*USES*/", &args.uses()),
            ("/*INCLUDES*/", &includes),
            ("/*SETUP*/", &args.setup()),
            ("/*EXPRESSIONS*/", &args.expressions()),
            ("/*TIMER*/", &args.timer()),
        ],
    )?;

    fs::remove_dir_all("target/criterion").ok();

    let mut cmdline = vec!["bench", "--bench", "timeit", "--", "--noplot"];
    if args.verbose {
        cmdline.push("--verbose");
    }
    process::Command::new("cargo").args(&cmdline).status()?;

    if args.cleanup {
        println!("Deleting cache directory.");
        fs::remove_dir_all(&base_dir)?;
    }
    Ok(())
}
