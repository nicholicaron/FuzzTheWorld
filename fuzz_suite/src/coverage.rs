use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io;
use walkdir::WalkDir;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub taken: bool,
    pub count: u64,
    pub condition: Option<String>, // For C/C++ conditions
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub called: u64,
}

#[derive(Debug, Clone)]
pub struct CoverageData {
    pub lines: HashSet<Location>,
    pub branches: HashMap<Location, BranchInfo>,
    pub functions: HashMap<String, FunctionInfo>,
    pub includes: HashSet<PathBuf>,  // Track included files
}

pub struct Coverage {
    config: super::CoverageConfig,
    data: CoverageData,
}

impl Coverage {
    pub fn new(config: super::CoverageConfig) -> Self {
        Coverage {
            config,
            data: CoverageData {
                lines: HashSet::new(),
                branches: HashMap::new(),
                functions: HashMap::new(),
                includes: HashSet::new(),
            },
        }
    }

    /// Process LLVM coverage data for C/C++ programs
    pub fn process_coverage(&mut self) -> io::Result<()> {
        // First, find the raw profile data
        let raw_profile = self.config.output_dir.join("default.profraw");
        if !raw_profile.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Raw profile data not found at {:?}", raw_profile)
            ));
        }


        // Merge coverage profiles
        let merge_status = Command::new("llvm-profdata")
            .arg("merge")
            .arg("-sparse")
            .arg(&raw_profile)
            .arg("-o")
            .arg(self.config.output_dir.join("coverage.profdata"))
            .status()?;

        if !merge_status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to merge profile data"
            ));
        }

        // Generate coverage information
        let output = Command::new("llvm-cov")
            .arg("show")
            .arg("--format=text")
            .arg("--show-expansions")  // Show macro expansions
            .arg("--show-branches")    // Show branch coverage
            .arg("--show-functions")   // Show function coverage
            .arg("--instr-profile")
            .arg(self.config.output_dir.join("coverage.profdata"))
            .arg(&self.config.source_dir)
            .output()?;

        self.parse_coverage_output(String::from_utf8_lossy(&output.stdout).as_ref())?;
        self.process_includes()?;
        Ok(())
    }

    /// Process included files to track their coverage
    fn process_includes(&mut self) -> io::Result<()> {
        for file in WalkDir::new(&self.config.source_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "h" || ext == "hpp"))
        {
            self.data.includes.insert(file.path().to_path_buf());
        }
        Ok(())
    }

    fn parse_coverage_output(&mut self, output: &str) -> io::Result<()> {
        for line in output.lines() {
            if let Some(coverage_info) = self.parse_coverage_line(line) {
                self.data.lines.insert(coverage_info);
            }
        }
        Ok(())
    }

    fn parse_coverage_line(&self, line: &str) -> Option<Location> {
        // Parse LLVM coverage output format for C/C++
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            Some(Location {
                file: PathBuf::from(parts[0]),
                line: parts[1].parse().ok()?,
                column: parts[2].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn get_coverage_data(&self) -> &CoverageData {
        &self.data
    }

    pub fn coverage_percentage(&self) -> f64 {
        let total_lines = self.count_total_lines();
        if total_lines == 0 {
            return 0.0;
        }
        (self.data.lines.len() as f64 / total_lines as f64) * 100.0
    }

    fn count_total_lines(&self) -> usize {
        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(&self.config.source_dir) {
            for entry in entries.filter_map(Result::ok) {
                if super::compiler::detect_language(entry.path().as_ref()).is_some() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        total += content.lines().count();
                    }
                }
            }
        }
        total
    }
}
