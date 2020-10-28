use argh::FromArgs;
use std::{
    env,
    fs::{self, File},
    io::{Error, Read as _, Write as _},
    process,
};

const BASE: &str = "timeit";
const CARGO_TOML: &str = include_str!("../templates/Cargo.toml");
const TIMEIT_EXPRESSION: &str = include_str!("../templates/expression.rs");
const TIMEIT_RS: &str = include_str!("../templates/timeit.rs");

#[derive(Debug, FromArgs)]
#[argh(description = "FIXME")]
struct Args {
    #[argh(
        option,
        short = 's',
        description = "code to be executed once before timing begins"
    )]
    setup: Option<String>,

    #[argh(
        option,
        short = 'd',
        description = "crate name and version to add to the dependencies section"
    )]
    dependency: Vec<String>,

    #[argh(
        option,
        short = 'i',
        description = "include the named file's contents in the source code"
    )]
    include: Vec<String>,

    #[argh(positional)]
    expression: Vec<String>,
}

impl Args {
    fn setup(&self) -> String {
        self.setup.clone().unwrap_or_default()
    }

    fn dependencies(&self) -> String {
        self.dependency.join("\n")
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

    fn expressions(&self) -> String {
        self.expression
            .iter()
            .map(|expression| TIMEIT_EXPRESSION.replace("@EXPRESSION@", &expression))
            .collect::<Vec<_>>()
            .join("\n")
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

fn main() -> Result<(), Error> {
    let args = argh::from_env::<Args>();
    if args.expression.is_empty() {
        eprintln!("Please specify at least one expression");
        process::exit(1);
    }

    let mut base_dir = dirs::cache_dir().expect("Could not determine cache directory");
    base_dir.push("rust-timeit");
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
            ("@SETUP@", &args.setup()),
            ("@EXPRESSIONS@", &args.expressions()),
            ("@INCLUDES@", &args.includes()?),
        ],
    )?;

    fs::remove_dir_all("target/criterion").ok();

    process::Command::new("cargo")
        .args(&["bench", "--bench", "timeit", "--", "--noplot"])
        .status()?;

    Ok(())
}
