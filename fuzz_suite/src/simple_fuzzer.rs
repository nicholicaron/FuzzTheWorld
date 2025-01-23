use rand::Rng;
use std::process::{Command, Output};
use std::string::String;

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Pass,
    Fail,
    Unresolved,
}

pub trait Runner {
    fn run(&self, input: &str) -> (Box<dyn std::any::Any>, Outcome);
}

pub struct PrintRunner;

impl Runner for PrintRunner {
    fn run(&self, input: &str) -> (Box<dyn std::any::Any>, Outcome) {
        println!("{}", input);
        (Box::new(input.to_string()), Outcome::Unresolved)
    }
}

pub struct ProgramRunner {
    program: String,
}

impl ProgramRunner {
    pub fn new(program: &str) -> Self {
        ProgramRunner {
            program: program.to_string(),
        }
    }

    fn run_process(&self, input: &str) -> std::io::Result<Output> {
        Command::new(&self.program)
            .arg(input)
            .output()
    }
}

impl Runner for ProgramRunner {
    fn run(&self, input: &str) -> (Box<dyn std::any::Any>, Outcome) {
        match self.run_process(input) {
            Ok(output) => {
                let outcome = if output.status.success() {
                    Outcome::Pass
                } else if output.status.code().is_none() {
                    Outcome::Fail
                } else {
                    Outcome::Unresolved
                };
                (Box::new(output), outcome)
            }
            Err(e) => (Box::new(e.to_string()), Outcome::Fail),
        }
    }
}

pub struct BinaryProgramRunner {
    program: String,
}

impl BinaryProgramRunner {
    pub fn new(program: &str) -> Self {
        BinaryProgramRunner {
            program: program.to_string(),
        }
    }

    fn run_process(&self, input: &str) -> std::io::Result<Output> {
        Command::new(&self.program)
            .arg(input)
            .output()
    }
}

impl Runner for BinaryProgramRunner {
    fn run(&self, input: &str) -> (Box<dyn std::any::Any>, Outcome) {
        match self.run_process(input) {
            Ok(output) => {
                let outcome = if output.status.success() {
                    Outcome::Pass
                } else if output.status.code().is_none() {
                    Outcome::Fail
                } else {
                    Outcome::Unresolved
                };
                (Box::new(output), outcome)
            }
            Err(e) => (Box::new(e.to_string()), Outcome::Fail),
        }
    }
}

pub trait Fuzzer {
    fn fuzz(&self) -> String;
    
    fn run(&self, runner: &dyn Runner) -> (Box<dyn std::any::Any>, Outcome) {
        runner.run(&self.fuzz())
    }
    
    fn runs(&self, runner: &dyn Runner, trials: usize) -> Vec<(Box<dyn std::any::Any>, Outcome)> {
        (0..trials).map(|_| self.run(runner)).collect()
    }
}

pub struct RandomFuzzer {
    min_length: usize,
    max_length: usize,
    char_start: u32,
    char_range: u32,
}

impl RandomFuzzer {
    pub fn new(min_length: usize, max_length: usize, char_start: u32, char_range: u32) -> Self {
        RandomFuzzer {
            min_length,
            max_length,
            char_start,
            char_range,
        }
    }
}

impl Default for RandomFuzzer {
    fn default() -> Self {
        RandomFuzzer {
            min_length: 10,
            max_length: 100,
            char_start: 32,
            char_range: 32,
        }
    }
}

impl Fuzzer for RandomFuzzer {
    fn fuzz(&self) -> String {
        let mut rng = rand::thread_rng();
        let string_length = rng.gen_range(self.min_length..=self.max_length);
        
        (0..string_length)
            .map(|_| {
                let char_code = rng.gen_range(self.char_start..self.char_start + self.char_range);
                char::from_u32(char_code).unwrap_or(' ')
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_fuzzer_output_length() {
        let fuzzer = RandomFuzzer::new(10, 20, 32, 32);
        let output = fuzzer.fuzz();
        assert!(output.len() >= 10 && output.len() <= 20);
    }

    #[test]
    fn test_random_fuzzer_character_range() {
        let fuzzer = RandomFuzzer::new(100, 100, 65, 26); // A-Z range
        let output = fuzzer.fuzz();
        for c in output.chars() {
            assert!(c >= 'A' && c <= 'Z');
        }
    }

    #[test]
    fn test_print_runner() {
        let runner = PrintRunner;
        let (result, outcome) = runner.run("test input");
        assert_eq!(outcome, Outcome::Unresolved);
        assert_eq!(result.downcast_ref::<String>().unwrap(), "test input");
    }

    #[test]
    fn test_multiple_runs() {
        let fuzzer = RandomFuzzer::default();
        let runner = PrintRunner;
        let results = fuzzer.runs(&runner, 5);
        assert_eq!(results.len(), 5);
    }
}
