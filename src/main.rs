use rand::Rng;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zkevm_fuzzer::fuzzer::FUZZERS;

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let all_fuzzers = FUZZERS.keys().copied().collect::<Vec<&'static str>>();
    // [(failed_count, total_count),...]
    let fuzzer_counter = (0..all_fuzzers.len())
        .map(|_| (AtomicUsize::new(0), AtomicUsize::new(0)))
        .collect::<Vec<(AtomicUsize, AtomicUsize)>>();
    info!("compiled with {} fuzzers", all_fuzzers.len());
    for fuzzer in all_fuzzers.iter() {
        info!("found fuzzer: {}", fuzzer);
    }

    let n_cpus = num_cpus::get();
    let use_cpus = if n_cpus == 1 { 1 } else { n_cpus - 1 };
    info!("using {} of {} CPUs", use_cpus, n_cpus);

    let stop_flag = Box::leak(Box::new(AtomicBool::new(false))) as &AtomicBool;

    ctrlc::set_handler(|| stop_flag.store(true, Ordering::Relaxed)).unwrap();

    let (failures_tx, failures_rx) = std::sync::mpsc::channel();

    thread::scope(|s| {
        for _ in 0..use_cpus {
            let all_fuzzers = all_fuzzers.as_slice();
            let fuzzer_counter = fuzzer_counter.as_slice();
            let failures_tx = failures_tx.clone();
            s.spawn(move || {
                let mut rng = rand::thread_rng();
                while !stop_flag.load(Ordering::Relaxed) {
                    let idx = rng.gen_range(0..all_fuzzers.len());
                    let fuzzer_name = all_fuzzers[idx];
                    let fuzzer = FUZZERS.get(fuzzer_name).unwrap();
                    let case = fuzzer.gen_test_case();
                    if let Err(_) = case.test_builder.run_catch() {
                        failures_tx
                            .send((fuzzer_name, serde_json::to_value(case.input).unwrap()))
                            .unwrap();
                        fuzzer_counter[idx].0.fetch_add(1, Ordering::SeqCst);
                    }
                    fuzzer_counter[idx].1.fetch_add(1, Ordering::SeqCst);
                }
            });
        }

        loop {
            let awake_time = Instant::now();
            let next_awake = awake_time
                .checked_add(std::time::Duration::from_secs(10))
                .unwrap();
            while let Ok((fuzzer_name, input)) = failures_rx.try_recv() {
                info!("failed fuzzer: {}, input: {:?}", fuzzer_name, input);
            }
            for (idx, (name, (failed, total))) in
            all_fuzzers.iter().zip(fuzzer_counter.iter()).enumerate()
            {
                info!(
                "{}: fuzzer: {}, failed: {}, total: {}",
                idx,
                name,
                failed.load(Ordering::Relaxed),
                total.load(Ordering::Relaxed)
            );
            }

            let go_sleep_time = Instant::now();
            if next_awake > go_sleep_time {
                thread::sleep(next_awake.duration_since(go_sleep_time));
            }
        }
    })
}
