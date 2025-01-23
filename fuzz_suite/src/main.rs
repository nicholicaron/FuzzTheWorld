use fuzz_suite::{
    Coverage, CoverageConfig, CompilerConfig,
    Language, compile_with_coverage, RandomFuzzer, Fuzzer
};
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::io::Write;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("C/C++ Program Fuzzer with Coverage Analysis");
    println!("==========================================");

    // Setup base directories
    let example_dir = PathBuf::from("examples/cgi_decode");
    let source_file = example_dir.join("src/cgi_decode.c");
    
    // Create necessary directories
    let bin_dir = example_dir.join("bin");
    let crashes_dir = example_dir.join("crashes");
    let coverage_dir = example_dir.join("coverage");
    let fuzz_dir = example_dir.join("fuzz_inputs");

    for dir in [&bin_dir, &crashes_dir, &coverage_dir, &fuzz_dir] {
        std::fs::create_dir_all(dir)?;
    }

    let output_file = bin_dir.join("cgi_decode");

    println!("Setting up fuzzing configuration...");
    
    // Configure compiler
    let compiler_config = CompilerConfig::new(Language::C)
        .with_flags(vec![
            "-DTESTING".to_string(),
            "-g".to_string(),
            "-fsanitize=address".to_string(),
        ]);

    // Configure coverage
    let coverage_config = CoverageConfig {
        output_dir: coverage_dir.clone(),
        source_dir: source_file.parent().unwrap().to_path_buf(),
        profile_data: coverage_dir.join("cgi_decode.profdata"),
        compiler: compiler_config,
    };

    // Set LLVM_PROFILE_FILE to store raw profile data in our coverage directory
    std::env::set_var(
        "LLVM_PROFILE_FILE",
        coverage_dir.join("default.profraw").to_str().unwrap()
    );

    println!("Compiling program with coverage instrumentation...");
    compile_with_coverage(&source_file, &output_file, &coverage_config)?;

    // Initialize fuzzer and coverage tracking
    let fuzzer = RandomFuzzer::new(1,10,32,95); // sample strings from printable ASCII range
    let mut coverage = Coverage::new(coverage_config);
    let mut cumulative_coverage = Vec::new();
    let mut crashes = Vec::new();

    println!("\nStarting fuzzing run with 100 inputs...");
    println!("----------------------------------------");

    // Run 100 fuzz tests
    for i in 0..100 {
        if i % 10 == 0 {
            println!("Processing input {}/100...", i);
        }

        let input = fuzzer.fuzz();

        // Run program with fuzzer input
        let output = Command::new(&output_file)
            .arg(&input)
            .output()?;
        
        // Check for crashes
        if !output.status.success() {
            // println!("🐛 Found crash with input {}!", i);
            let crash_file = crashes_dir.join(format!("crash_{}.txt", i));
            let mut file = File::create(&crash_file)?;
            file.write_all(input.as_bytes())?;
            crashes.push(input);
        }
        
        // Process coverage
        coverage.process_coverage()?;
        cumulative_coverage.push((i + 1, coverage.coverage_percentage()));
    }

    println!("\nGenerating coverage reports...");
    println!("-----------------------------");

    // Transform coverage data for plotting
    let coverage_percentages: Vec<f64> = cumulative_coverage.iter()
        .map(|(_, coverage)| *coverage)
        .collect();

    // Generate coverage visualization
    fuzz_suite::plot_coverage(
        &coverage_percentages,
        &coverage_dir.join("coverage_over_time.png"),
    )?;

    // Generate coverage report
    let coverage_data = coverage.get_coverage_data();
    fuzz_suite::generate_lcov(
        coverage_data,
        &coverage_dir.join("coverage.lcov"),
    )?;

    // Print summary
    println!("\nFuzzing Summary");
    println!("==============");
    println!("Final coverage: {:.2}%", coverage.coverage_percentage());
    println!("Found {} crashes", crashes.len());
    println!("\nArtifacts written to:");
    println!("- Coverage plot: {}", coverage_dir.join("coverage_over_time.png").display());
    println!("- Coverage report: {}", coverage_dir.join("coverage.lcov").display());
    if !crashes.is_empty() {
        println!("- Crash files: {}", crashes_dir.display());
    }

    Ok(())
}
