use std::collections::{HashMap,HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use super::coverage::{CoverageData, Location};

#[derive(Debug)]
pub struct CoverageReport {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub line_coverage: f64,
}

impl CoverageReport {
    pub fn new(coverage_data: &CoverageData, total_lines: usize) -> Self {
        let covered_lines = coverage_data.lines.len();
        let line_coverage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let branch_coverage = coverage_data.branches.values()
            .filter(|info| info.taken)
            .count() as f64 / coverage_data.branches.len() as f64 * 100.0;

        CoverageReport {
            total_lines,
            covered_lines,
            branch_coverage,
            function_coverage: 100.0, // Placeholder until we implement function tracking
            line_coverage,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "Coverage Report:\n\
             Total Lines: {}\n\
             Covered Lines: {}\n\
             Line Coverage: {:.2}%\n\
             Branch Coverage: {:.2}%\n\
             Function Coverage: {:.2}%\n",
            self.total_lines,
            self.covered_lines,
            self.line_coverage,
            self.branch_coverage,
            self.function_coverage
        )
    }
}

pub fn generate_lcov(
    coverage_data: &CoverageData,
    output_path: &Path,
) -> io::Result<()> {
    let mut file = File::create(output_path)?;
    
    // Group coverage data by file
    let mut file_coverage: HashMap<_, Vec<_>> = HashMap::new();
    for location in &coverage_data.lines {
        file_coverage
            .entry(&location.file)
            .or_default()
            .push(location);
    }

    for (file_path, locations) in file_coverage {
        // Write file section
        writeln!(file, "SF:{}", file_path.display())?;

        // Write function coverage (placeholder for now)
        writeln!(file, "FNF:0")?;
        writeln!(file, "FNH:0")?;

        // Write line coverage
        for location in locations {
            writeln!(file, "DA:{},1", location.line)?;
        }

        // Write branch coverage
        let file_branches: Vec<_> = coverage_data.branches
            .iter()
            .filter(|(loc, _)| loc.file == *file_path)
            .collect();

        for (location, info) in file_branches {
            writeln!(
                file,
                "BRDA:{},{},{},{}",
                location.line,
                0, // branch number (placeholder)
                0, // block number (placeholder)
                if info.taken { info.count } else { 0 }
            )?;
        }

        // Write end of record
        writeln!(file, "end_of_record")?;
    }

    Ok(())
}

/// Format source code with coverage information
pub fn format_source_with_coverage(
    source: &str,
    coverage_data: &CoverageData,
    file_path: &Path,
) -> String {
    let mut result = String::new();
    let covered_lines: HashSet<u32> = coverage_data.lines
        .iter()
        .filter(|loc| loc.file == file_path)
        .map(|loc| loc.line)
        .collect();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num as u32 + 1;
        if covered_lines.contains(&line_num) {
            result.push_str(" ");
        } else {
            result.push_str("# ");
        }
        result.push_str(&format!("{:2} {}\n", line_num, line));
    }

    result
}
