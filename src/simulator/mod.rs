mod client;
mod confidence_interval;
mod exponential_time_generator;
mod queue;
mod sample_accumulators;

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;

use confidence_interval::ConfidenceInterval;
use queue::{Queue, N, NQ, T, W, X};
use sample_accumulators::sample::Sample;

pub(crate) use queue::QueuePolicy;

fn statistics_hash_map(rounds_count: usize) -> BTreeMap<String, Sample> {
    let mut statistics = BTreeMap::new();
    statistics.insert(N.to_string(), Sample::new(rounds_count));
    statistics.insert(T.to_string(), Sample::new(rounds_count));
    statistics.insert(X.to_string(), Sample::new(rounds_count));
    statistics
}

pub fn simulator(
    rho: f64,
    transient_phase_size: usize,
    round_size: usize,
    rounds_count: usize,
    queue_policy: QueuePolicy,
    seed: u64,
) {
    let mut means_statistics = statistics_hash_map(rounds_count);
    let mut variances_statistics = statistics_hash_map(rounds_count);

    let mut w_mean_statistics = Sample::new(rounds_count);
    let mut w_variance_statistics = Sample::new(rounds_count);

    let mut nq_mean_statistics = Sample::new(rounds_count);
    let mut nq_variance_statistics = Sample::new(rounds_count);

    let mut queue = Queue::new(rho, queue_policy, seed);

    queue.run_one_simulation_round(transient_phase_size);

    let progress_bar = ProgressBar::new(rounds_count as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.white} [{elapsed_precise}] [{bar:40.red/green}] {percent:>3}% {pos:>4}/{len} ({eta_precise})")
            .progress_chars("🔥💧"),
    );
    for _ in 0..rounds_count {
        progress_bar.inc(1);
        let (samples, stochastic_process_samples) = queue.run_one_simulation_round(round_size);
        for (name, sample) in samples {
            if W == name {
                w_mean_statistics.append(sample.mean());
                w_variance_statistics.append(sample.variance());
            } else {
                means_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.mean());
                variances_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.variance());
            }
        }
        for (name, sample) in stochastic_process_samples {
            if NQ == name {
                nq_mean_statistics.append(sample.mean());
                nq_variance_statistics.append(sample.variance());
            } else {
                means_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.mean());
                variances_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.variance());
            }
        }
    }
    progress_bar.finish_with_message("Finalizado");

    println!(
        "\nTotal de fregueses {} com política {:#?} e ρ = {}\n",
        round_size, queue_policy, rho
    );
    for (name, statistic) in &means_statistics {
        println!("E[{}] = {:0.5}", name, statistic.mean())
    }
    for (name, statistic) in &variances_statistics {
        println!("V({}) = {:0.5}", name, statistic.mean())
    }
    println!();

    // Item a)
    let mean_w_ci = w_mean_statistics.t_student_95percent();
    println!(
        "E[W] = {:0.5}\\n\tIC T-Student:  L(0.05) = {:0.5}; Center = {:0.5}; U(0.05) = {:0.5}; \
         Precision = {:0.5}%",
        w_mean_statistics.mean(),
        mean_w_ci.lower_bound(),
        mean_w_ci.center(),
        mean_w_ci.upper_bound(),
        100.0 - mean_w_ci.precision()
    );

    // Item b)
    let ts_ci_w = w_variance_statistics.t_student_95percent();
    let c2_ci_w = w_variance_statistics.chi_square_95percent(w_variance_statistics.mean());
    println!(
        "V(W) = {:0.5}\n\tIC T-Student:  L(0.05) = {:0.5}; Center = {:0.5}; U(0.05) = {:0.5}; \
    Precision = {:0.5}%\n\tIC Chi-Square: L(0.05) = {:0.5}; Center = {:0.5}; U(0.05) = {:0.5}; \
    Precision = {:0.5}%",
        w_variance_statistics.mean(),
        ts_ci_w.lower_bound(),
        ts_ci_w.center(),
        ts_ci_w.upper_bound(),
        100.0 - ts_ci_w.precision(),
        c2_ci_w.lower_bound(),
        c2_ci_w.center(),
        c2_ci_w.upper_bound(),
        100.0 - c2_ci_w.precision()
    );

    // Item c)
    let mean_nq_ci = nq_mean_statistics.t_student_95percent();
    println!(
        "E[Nq] = {:0.5}\n\tIC T-Student:  L(0.05) = {:0.5}; \
         Center = {:0.5}; U(0.05) = {:0.5}; Precision = {:0.5}%",
        nq_mean_statistics.mean(),
        mean_nq_ci.lower_bound(),
        mean_nq_ci.center(),
        mean_nq_ci.upper_bound(),
        100.0 - mean_nq_ci.precision()
    );

    // Item d)
    let ts_ci_nq = nq_variance_statistics.t_student_95percent();
    let c2_ci_nq = nq_variance_statistics.chi_square_95percent(nq_variance_statistics.mean());
    println!(
        "V(Nq) = {:0.5}\n\tIC T-Student:  L(0.05) = {:0.5}; Center = {:0.5}; U(0.05) = {:0.5}; \
    Precision = {:0.5}%\n\tIC Chi-Square: L(0.05) = {:0.5}; Center = {:0.5}; U(0.05) = {:0.5}; \
    Precision = {:0.5}%",
        nq_variance_statistics.mean(),
        ts_ci_nq.lower_bound(),
        ts_ci_nq.center(),
        ts_ci_nq.upper_bound(),
        100.0 - ts_ci_nq.precision(),
        c2_ci_nq.lower_bound(),
        c2_ci_nq.center(),
        c2_ci_nq.upper_bound(),
        100.0 - c2_ci_nq.precision()
    );

    if mean_w_ci.precision() > 0.05 {
        println!(
            "Precisão do IC de E[W] = {:0.5}% não é suficiente\
             \nRodando agora para {} clientes",
            100.0 - mean_w_ci.precision(),
            round_size + 100
        );
        return simulator(
            rho,
            transient_phase_size,
            round_size + 100,
            rounds_count,
            queue_policy,
            seed,
        );
    }

    if mean_nq_ci.precision() > 0.05 {
        println!(
            "Precisão do IC de E[Nq] = {:0.5}% não é suficiente\
             \nRodando agora para {} clientes",
            100.0 - mean_nq_ci.precision(),
            round_size + 100
        );
        return simulator(
            rho,
            transient_phase_size,
            round_size + 100,
            rounds_count,
            queue_policy,
            seed,
        );
    }

    if !ConfidenceInterval::check_convergence(c2_ci_w, ts_ci_w) {
        println!(
            "Os intervalos de confiança para V(W) com T-Student e Chi-Square não convergem\
             \nRodando agora para {} clientes\n",
            round_size + 100
        );
        return simulator(
            rho,
            transient_phase_size,
            round_size + 100,
            rounds_count,
            queue_policy,
            seed,
        );
    }

    if !ConfidenceInterval::check_convergence(c2_ci_nq, ts_ci_nq) {
        println!(
            "Os intervalos de confiança para V(Nq) com T-Student e Chi-Square não convergem\
             \nRodando agora para {} clientes\n",
            round_size + 100
        );
        return simulator(
            rho,
            transient_phase_size,
            round_size + 100,
            rounds_count,
            queue_policy,
            seed,
        );
    }
}
