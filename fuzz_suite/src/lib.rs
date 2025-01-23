// lib.rs
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::time::SystemTime;

mod coverage;
mod coverage_visualization;
mod coverage_report;
mod compiler;
mod simple_fuzzer;

pub use coverage::{Coverage, CoverageData, Location};
pub use coverage_visualization::plot_coverage;
pub use coverage_report::{generate_lcov, CoverageReport};
pub use compiler::{CompilerConfig, Language};
pub use simple_fuzzer::{RandomFuzzer, Fuzzer};


#[derive(Debug, Clone)]
pub struct CoverageConfig {
    pub output_dir: PathBuf,
    pub source_dir: PathBuf,
    pub profile_data: PathBuf,
    pub compiler: CompilerConfig,
}

impl Default for CoverageConfig {
    fn default() -> Self {
        CoverageConfig {
            output_dir: PathBuf::from("coverage"),
            source_dir: PathBuf::from("src"),
            profile_data: PathBuf::from("coverage.profdata"),
            compiler: CompilerConfig::default(),
        }
    }
}

/// Initialize coverage tracking for C/C++ programs
pub fn init_coverage(config: &CoverageConfig) -> io::Result<()> {
    std::fs::create_dir_all(&config.output_dir)?;
    std::env::set_var("LLVM_PROFILE_FILE", config.profile_data.to_str().unwrap());
    Ok(())
}

/// Compile a C/C++ program with coverage instrumentation
pub fn compile_with_coverage(
    source_file: &Path,
    output_file: &Path,
    config: &CoverageConfig,
) -> io::Result<()> {
    config.compiler.compile_with_coverage(source_file, output_file)
}

/// Save a crashing input to a file
pub fn save_crash(
    input: &str,
    crashes_dir: &Path,
    identifier: &str,
) -> io::Result<PathBuf> {
    std::fs::create_dir_all(crashes_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let crash_file = crashes_dir.join(format!("crash_{}_{}.txt", identifier, timestamp));
    let mut file = File::create(&crash_file)?;
    file.write_all(input.as_bytes())?;
    Ok(crash_file)
}
