//! Implementation of the `wasm-pack build` command.

use crate::wasm_opt;
use binary_install::Cache;
use bindgen;
use build;
use cache;
use command::utils::{create_pkg_dir, get_crate_path};
use emoji;
use failure::Error;
use install::{self, InstallMode, Tool};
use license;
use lockfile::Lockfile;
use log::info;
use manifest;
use readme;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use PBAR;

/// Everything required to configure and run the `wasm-pack build` command.
#[allow(missing_docs)]
pub struct Build {
    pub crate_path: PathBuf,
    pub crate_data: manifest::CrateData,
    pub scope: Option<String>,
    pub disable_dts: bool,
    pub reference_types: bool,
    pub target: Target,
    pub profile: BuildProfile,
    pub mode: InstallMode,
    pub out_dir: PathBuf,
    pub out_name: Option<String>,
    pub bindgen: Option<install::Status>,
    pub cache: Cache,
    pub extra_options: Vec<String>,
}

/// What sort of output we're going to be generating and flags we're invoking
/// `wasm-bindgen` with.
#[derive(Clone, Copy, Debug)]
pub enum Target {
    /// Default output mode or `--target bundler`, indicates output will be
    /// used with a bundle in a later step.
    Bundler,
    /// Correspond to `--target web` where the output is natively usable as an
    /// ES module in a browser and the wasm is manually instantiated.
    Web,
    /// Correspond to `--target nodejs` where the output is natively usable as
    /// a Node.js module loaded with `require`.
    Nodejs,
    /// Correspond to `--target no-modules` where the output is natively usable
    /// in a browser but pollutes the global namespace and must be manually
    /// instantiated.
    NoModules,
}

impl Default for Target {
    fn default() -> Target {
        Target::Bundler
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Target::Bundler => "bundler",
            Target::Web => "web",
            Target::Nodejs => "nodejs",
            Target::NoModules => "no-modules",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "bundler" | "browser" => Ok(Target::Bundler),
            "web" => Ok(Target::Web),
            "nodejs" => Ok(Target::Nodejs),
            "no-modules" => Ok(Target::NoModules),
            _ => bail!("Unknown target: {}", s),
        }
    }
}

/// The build profile controls whether optimizations, debug info, and assertions
/// are enabled or disabled.
#[derive(Clone, Copy, Debug)]
pub enum BuildProfile {
    /// Enable assertions and debug info. Disable optimizations.
    Dev,
    /// Enable optimizations. Disable assertions and debug info.
    Release,
    /// Enable optimizations and debug info. Disable assertions.
    Profiling,
}

/// Everything required to configure and run the `wasm-pack build` command.
#[derive(Debug, StructOpt)]
pub struct BuildOptions {
    /// The path to the Rust crate. If not set, searches up the path from the current directory.
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[structopt(long = "scope", short = "s")]
    pub scope: Option<String>,

    #[structopt(long = "mode", short = "m", default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal, force]
    pub mode: InstallMode,

    #[structopt(long = "no-typescript")]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[structopt(long = "reference-types")]
    /// Enable support for reference types. This feature is hoped to enable more efficient
    /// communication between the host (JS) and the wasm module, but not yet all browsers support it.
    pub reference_types: bool,

    #[structopt(long = "target", short = "t", default_value = "bundler")]
    /// Sets the target environment. [possible values: bundler, nodejs, web, no-modules]
    pub target: Target,

    #[structopt(long = "debug")]
    /// Deprecated. Renamed to `--dev`.
    pub debug: bool,

    #[structopt(long = "dev")]
    /// Create a development build. Enable debug info, and disable
    /// optimizations.
    pub dev: bool,

    #[structopt(long = "release")]
    /// Create a release build. Enable optimizations and disable debug info.
    pub release: bool,

    #[structopt(long = "profiling")]
    /// Create a profiling build. Enable optimizations and debug info.
    pub profiling: bool,

    #[structopt(long = "out-dir", short = "d", default_value = "pkg")]
    /// Sets the output directory with a relative path.
    pub out_dir: String,

    #[structopt(long = "out-name")]
    /// Sets the output file names. Defaults to package name.
    pub out_name: Option<String>,

    #[structopt(last = true)]
    /// List of extra options to pass to `cargo build`
    pub extra_options: Vec<String>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            path: None,
            scope: None,
            mode: InstallMode::default(),
            disable_dts: false,
            reference_types: false,
            target: Target::default(),
            debug: false,
            dev: false,
            release: false,
            profiling: false,
            out_dir: String::new(),
            out_name: None,
            extra_options: Vec::new(),
        }
    }
}

type BuildStep = fn(&mut Build) -> Result<(), Error>;

impl Build {
    /// Construct a build command from the given options.
    pub fn try_from_opts(build_opts: BuildOptions) -> Result<Self, Error> {
        let crate_path = get_crate_path(build_opts.path)?;
        let crate_data = manifest::CrateData::new(&crate_path, build_opts.out_name.clone())?;
        let out_dir = crate_path.join(PathBuf::from(build_opts.out_dir));

        let dev = build_opts.dev || build_opts.debug;
        let profile = match (dev, build_opts.release, build_opts.profiling) {
            (false, false, false) | (false, true, false) => BuildProfile::Release,
            (true, false, false) => BuildProfile::Dev,
            (false, false, true) => BuildProfile::Profiling,
            // Unfortunately, `structopt` doesn't expose clap's `conflicts_with`
            // functionality yet, so we have to implement it ourselves.
            _ => bail!("Can only supply one of the --dev, --release, or --profiling flags"),
        };

        Ok(Build {
            crate_path,
            crate_data,
            scope: build_opts.scope,
            disable_dts: build_opts.disable_dts,
            reference_types: build_opts.reference_types,
            target: build_opts.target,
            profile,
            mode: build_opts.mode,
            out_dir,
            out_name: build_opts.out_name,
            bindgen: None,
            cache: cache::get_wasm_pack_cache()?,
            extra_options: build_opts.extra_options,
        })
    }

    /// Configures the global binary cache used for this build
    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }

    /// Execute this `Build` command.
    pub fn run(&mut self) -> Result<(), Error> {
        let process_steps = Build::get_process_steps(self.mode);

        let started = Instant::now();

        for (_, process_step) in process_steps {
            process_step(self)?;
        }

        let duration = crate::command::utils::elapsed(started.elapsed());
        info!("Done in {}.", &duration);
        info!(
            "Your wasm pkg is ready to publish at {}.",
            self.out_dir.display()
        );

        PBAR.info(&format!("{} Done in {}", emoji::SPARKLE, &duration));

        PBAR.info(&format!(
            "{} Your wasm pkg is ready to publish at {}.",
            emoji::PACKAGE,
            self.out_dir.display()
        ));
        Ok(())
    }

    fn get_process_steps(mode: InstallMode) -> Vec<(&'static str, BuildStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                let mut steps: Vec<(&'static str, BuildStep)> = Vec::new();
                    $(steps.push((stringify!($name), Build::$name));)*
                        steps
                    }
                };
            ($($name:ident,)*) => (steps![$($name),*])
        }
        let mut steps = Vec::new();
        match &mode {
            InstallMode::Force => {}
            _ => {
                steps.extend(steps![
                    step_check_rustc_version,
                    step_check_crate_config,
                    step_check_for_wasm_target,
                ]);
            }
        }
        steps.extend(steps![
            step_build_wasm,
            step_create_dir,
            step_copy_readme,
            step_copy_license,
            step_install_wasm_bindgen,
            step_run_wasm_bindgen,
            step_run_wasm_opt,
            step_create_json,
        ]);
        steps
    }

    fn step_check_rustc_version(&mut self) -> Result<(), Error> {
        info!("Checking rustc version...");
        let version = build::check_rustc_version()?;
        let msg = format!("rustc version is {}.", version);
        info!("{}", &msg);
        Ok(())
    }

    fn step_check_crate_config(&mut self) -> Result<(), Error> {
        info!("Checking crate configuration...");
        self.crate_data.check_crate_config()?;
        info!("Crate is correctly configured.");
        Ok(())
    }

    fn step_check_for_wasm_target(&mut self) -> Result<(), Error> {
        info!("Checking for wasm-target...");
        build::wasm_target::check_for_wasm32_target()?;
        info!("Checking for wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self) -> Result<(), Error> {
        info!("Building wasm...");
        build::cargo_build_wasm(&self.crate_path, self.profile, &self.extra_options)?;

        info!(
            "wasm built at {:#?}.",
            &self
                .crate_path
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
        );
        Ok(())
    }

    fn step_create_dir(&mut self) -> Result<(), Error> {
        info!("Creating a pkg directory...");
        create_pkg_dir(&self.out_dir)?;
        info!("Created a pkg directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self) -> Result<(), Error> {
        self.crate_data.write_package_json(
            &self.out_dir,
            &self.scope,
            self.disable_dts,
            self.target,
        )?;
        info!(
            "Wrote a package.json at {:#?}.",
            &self.out_dir.join("package.json")
        );
        Ok(())
    }

    fn step_copy_readme(&mut self) -> Result<(), Error> {
        info!("Copying readme from crate...");
        readme::copy_from_crate(&self.crate_path, &self.out_dir)?;
        info!("Copied readme from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_copy_license(&mut self) -> Result<(), failure::Error> {
        info!("Copying license from crate...");
        license::copy_from_crate(&self.crate_data, &self.crate_path, &self.out_dir)?;
        info!("Copied license from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self) -> Result<(), failure::Error> {
        info!("Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_data)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;
        info!("Installing wasm-bindgen-cli...");
        let bindgen = install::download_prebuilt_or_cargo_install(
            Tool::WasmBindgen,
            &self.cache,
            &bindgen_version,
            self.mode.install_permitted(),
        )?;
        self.bindgen = Some(bindgen);
        info!("Installing wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self) -> Result<(), Error> {
        info!("Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_data,
            &self.bindgen.as_ref().unwrap(),
            &self.out_dir,
            &self.out_name,
            self.disable_dts,
            self.reference_types,
            self.target,
            self.profile,
        )?;
        info!("wasm bindings were built at {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_run_wasm_opt(&mut self) -> Result<(), Error> {
        if self.reference_types {
            return Ok(());
        }
        let args = match self
            .crate_data
            .configured_profile(self.profile)
            .wasm_opt_args()
        {
            Some(args) => args,
            None => return Ok(()),
        };
        info!("executing wasm-opt with {:?}", args);
        wasm_opt::run(
            &self.cache,
            &self.out_dir,
            &args,
            self.mode.install_permitted(),
        ).map_err(|e| {
            format_err!(
                "{}\nTo disable `wasm-opt`, add `wasm-opt = false` to your package metadata in your `Cargo.toml`.", e
            )
        })
    }
}
