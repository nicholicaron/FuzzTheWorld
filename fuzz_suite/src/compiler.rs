use std::path::Path;
use std::process::Command;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    C,
    Cpp,
}

impl Language {
    fn compiler_name(&self) -> &'static str {
        match self {
            Language::C => "clang",
            Language::Cpp => "clang++",
        }
    }

    fn from_extension(ext: &str) -> Option<Language> {
        match ext {
            "c" => Some(Language::C),
            "cpp" | "cc" | "cxx" => Some(Language::Cpp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub language: Language,
    pub compiler_path: String,
    pub optimization_level: String,
    pub extra_flags: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            language: Language::C,
            compiler_path: "clang".to_string(),
            optimization_level: "-O0".to_string(),
            extra_flags: vec![],
        }
    }
}

impl CompilerConfig {
    pub fn new(language: Language) -> Self {
        CompilerConfig {
            language,
            compiler_path: language.compiler_name().to_string(),
            ..Default::default()
        }
    }

    /// Compile a source file with coverage instrumentation
    pub fn compile_with_coverage(
        &self,
        source_file: &Path,
        output_file: &Path,
    ) -> io::Result<()> {
        let status = Command::new(&self.compiler_path)
            .arg(source_file)
            .arg("-o")
            .arg(output_file)
            .arg(self.optimization_level.as_str())
            .arg("--coverage")  // Enable gcov/llvm coverage
            .arg("-fprofile-instr-generate") // LLVM coverage instrumentation
            .arg("-fcoverage-mapping")       // Enable coverage mapping
            .args(&self.extra_flags)
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Compilation failed",
            ));
        }

        Ok(())
    }

    /// Add compiler flags
    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.extra_flags.extend(flags);
        self
    }

    /// Set optimization level
    pub fn with_optimization(mut self, level: &str) -> Self {
        self.optimization_level = level.to_string();
        self
    }
}

pub fn detect_language(source_file: &Path) -> Option<Language> {
    source_file
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(Language::from_extension)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            detect_language(Path::new("test.c")),
            Some(Language::C)
        );
        assert_eq!(
            detect_language(Path::new("test.cpp")),
            Some(Language::Cpp)
        );
        assert_eq!(
            detect_language(Path::new("test.txt")),
            None
        );
    }

    #[test]
    fn test_compiler_config() {
        let config = CompilerConfig::new(Language::Cpp)
            .with_optimization("-O2")
            .with_flags(vec!["-Wall".to_string()]);

        assert_eq!(config.language, Language::Cpp);
        assert_eq!(config.optimization_level, "-O2");
        assert!(config.extra_flags.contains(&"-Wall".to_string()));
    }

    #[test]
    fn test_compile_c_program() -> io::Result<()> {
        let dir = tempdir()?;
        let source_path = dir.path().join("test.c");
        let output_path = dir.path().join("test");

        // Create a simple C program
        let mut file = File::create(&source_path)?;
        writeln!(file, r#"
            #include <stdio.h>
            int main() {{
                printf("Hello, World!\n");
                return 0;
            }}
        "#)?;

        let config = CompilerConfig::new(Language::C);
        config.compile_with_coverage(&source_path, &output_path)?;

        assert!(output_path.exists());
        Ok(())
    }
}
