// Goal: Find an infinite recursion bug in XPDF 3.02
// CVE-2019-13288

use libafl::prelude::*;
use libafl_bolts::current_nanos;
use libafl_bolts::rands::StdRand;
use libafl_bolts::shmem::StdShMemProvider;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    // Current testcases of PDFs. In memory for faster access, files read as bytes
    let corpus_dirs = vec![PathBuf::from("./corpus")];
    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    // Save testcases that cause the program to timeout
    let timeouts_corpus =
        OnDiskCorpus::new(PathBuf::from("./timeouts")).expect("Could not create a timeouts corpus");

    // Save the runtime of each example as metadata
    let time_observer = TimeObserver::new("time");

    // Define shared memory mappings to enable fuzzer to keep track of coverage map
    const MAP_SIZE: usize = 65536;
    let mut shmem = StdShMemProvider::new().unwrap().new_map(MAP_SIZE).unwrap();
    // Save shared memory id to env so the executor knows about it
    shmem
        .write_to_env("__AFL_SHM_ID")
        .expect("Couldn't write shared memory ID");
    let mut shmem_map = shmem.as_mut_slice();

    let edges_observer =
        unsafe { HitcountsMapObserver::new(StdMapObserver::new("shared_mem", shmem_buf)) };

    // Keep track of interesting testcases -- If the current testcase's input triggered a new code
    // path in the coverage map, save that input to the corpus
    // Track indices, but do not track novelties
    let mut feedback = feedback_or!(
        MaxMapFeedback::tracking(&edges_observer, true, false),
        TimeFeedback::with_observer(&time_observer)
    );

    // If the given input triggered a new code path in the coverage map, AND, if the time to
    // execute the fuzz case with the current input results in a timeout, our testcase meets our
    // objetive
    let mut objective =
        feedback_and_fast!(TimeoutFeedback::new(), MaxMapFeedback::new(&edges_observer));

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        input_corpus,
        timeouts_corpus,
        &mut feedback,
        &mut objective,
    );

    let monitor = SimpleMonitor::new(|s| println!("{s}"));

    // Handle the various events generated during the fuzzing loop -- e.g. finding an interesting
    // testcase, updating the Monitor component, logging, etc.
    let mut mgr = SimpleEventManager::new(state);

    // Define strategy used to supply our fuzzer's request to the Corpus for a new testcase
    // Prioritize quick/small testcases that exercise all of the entries registered in teh coverage
    // map's metadata
    let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // fork_server defines executor builder that spawns child processes to fuzz
    let fork_server = ForkserverExecutor::builder()
        .program("./xpdf/install/bin/pdftotext")
        .parse_afl_cmdline(["@@"])
        .coverage_map_size(MAP_SIZE)
        .build(tuple_list!(time_observer, edges_observer))?;

    let timeout = Duration::from_secs(5);
    // ./pdftotext @@
    let mut executor = TimeoutForkserverExecutor::new(fork_server, timeout).unwrap();

    // Mutate the input between executions. Range of mutations = havoc mutations.
    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    // Actually running the fuzzer
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}
