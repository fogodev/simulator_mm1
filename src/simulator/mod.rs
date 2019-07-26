mod client;
mod confidence_interval;
mod exponential_time_generator;
mod queue;
mod sample_accumulators;
mod statistics_output_files;

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;

use confidence_interval::ConfidenceInterval;
use queue::{Queue, N, NQ, T, W, X};
use sample_accumulators::sample::Sample;

use crate::simulator::statistics_output_files::write_csv_file;
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
        "\nTotal de fregueses {} com política {:?} e ρ = {}\n",
        round_size, queue_policy, rho
    );

    let means_n_t_x = [
        means_statistics["N"].mean(),
        means_statistics["T"].mean(),
        means_statistics["X"].mean(),
    ];
    println!(
        "Sample Means:\n\tE[N] = {:0.5}\tE[T] = {:0.5}\tE[X] = {:0.5}",
        means_n_t_x[0], means_n_t_x[1], means_n_t_x[2],
    );

    let variances_n_t_x = [
        variances_statistics["N"].variance(),
        variances_statistics["T"].variance(),
        variances_statistics["X"].variance(),
    ];
    println!(
        "Sample Variances:\n\tV(N) = {:0.5}\tV(T) = {:0.5}\tV(X) = {:0.5}\n",
        variances_n_t_x[0], variances_n_t_x[1], variances_n_t_x[2],
    );

    // Item a)
    let mean_w_ci = w_mean_statistics.t_student_95percent();
    let mean_and_ic_w = extract_statistics_and_ci_slice(w_mean_statistics.mean(), &mean_w_ci);
    println!(
        "Sample Mean and Confidence Interval:\n\tE[W] = {:0.5}\n\t\tIC T-Student:\tL(0.05) = {:0.5};\
         \tCenter = {:0.5}; \tU(0.05) = {:0.5}; \tPrecision = {:0.5}%",
        mean_and_ic_w[0],
        mean_and_ic_w[1],
        mean_and_ic_w[2],
        mean_and_ic_w[3],
        100.0 - mean_and_ic_w[4],
    );

    // Item b)
    let w_variance = w_variance_statistics.mean();
    let ts_ci_w = w_variance_statistics.t_student_95percent();
    let c2_ci_w = w_variance_statistics.chi_square_95percent(w_variance);
    let variance_and_ic_t_student_chi_square_w =
        extract_statistics_and_2cis_slice(w_variance, &ts_ci_w, &c2_ci_w);
    println!(
        "Sample Variance and Confidence Interval:\n\tV(W) = {:0.5}\n\t\tIC T-Student:\
         \tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\tPrecision = {:0.5}%\
         \n\t\tIC Chi-Square:\tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\
         \tPrecision = {:0.5}%",
        variance_and_ic_t_student_chi_square_w[0],
        variance_and_ic_t_student_chi_square_w[1],
        variance_and_ic_t_student_chi_square_w[2],
        variance_and_ic_t_student_chi_square_w[3],
        100.0 - variance_and_ic_t_student_chi_square_w[4],
        variance_and_ic_t_student_chi_square_w[5],
        variance_and_ic_t_student_chi_square_w[6],
        variance_and_ic_t_student_chi_square_w[7],
        100.0 - variance_and_ic_t_student_chi_square_w[8],
    );

    // Item c)
    let mean_nq_ci = nq_mean_statistics.t_student_95percent();
    let mean_and_ic_nq = extract_statistics_and_ci_slice(nq_mean_statistics.mean(), &mean_nq_ci);
    println!(
        "Sample Mean and Confidence Interval:\n\tE[Nq] = {:0.5}\n\t\tIC T-Student:\tL(0.05) = {:0.5};\
         \tCenter = {:0.5}; \tU(0.05) = {:0.5}; \tPrecision = {:0.5}%",
        mean_and_ic_nq[0],
        mean_and_ic_nq[1],
        mean_and_ic_nq[2],
        mean_and_ic_nq[3],
        mean_and_ic_nq[4],
    );

    // Item d)
    let nq_variance = nq_variance_statistics.mean();
    let ts_ci_nq = nq_variance_statistics.t_student_95percent();
    let c2_ci_nq = nq_variance_statistics.chi_square_95percent(nq_variance);
    let variance_and_ic_t_student_chi_square_nq =
        extract_statistics_and_2cis_slice(nq_variance, &ts_ci_nq, &c2_ci_nq);
    println!(
        "Sample Variance and Confidence Interval:\n\tV(Nq) = {:0.5}\n\t\tIC T-Student:\
         \tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\tPrecision = {:0.5}%\
         \n\t\tIC Chi-Square:\tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\
         \tPrecision = {:0.5}%",
        variance_and_ic_t_student_chi_square_nq[0],
        variance_and_ic_t_student_chi_square_nq[1],
        variance_and_ic_t_student_chi_square_nq[2],
        variance_and_ic_t_student_chi_square_nq[3],
        100.0 - variance_and_ic_t_student_chi_square_nq[4],
        variance_and_ic_t_student_chi_square_nq[5],
        variance_and_ic_t_student_chi_square_nq[6],
        variance_and_ic_t_student_chi_square_nq[7],
        100.0 - variance_and_ic_t_student_chi_square_nq[8],
    );

    write_csv_file(
        rho,
        round_size,
        transient_phase_size,
        queue_policy,
        &means_n_t_x,
        &variances_n_t_x,
        &mean_and_ic_w,
        &variance_and_ic_t_student_chi_square_w,
        &mean_and_ic_nq,
        &variance_and_ic_t_student_chi_square_nq,
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

fn extract_statistics_and_ci_slice(statistic: f64, ci: &ConfidenceInterval) -> [f64; 5] {
    [
        statistic,
        ci.lower_bound(),
        ci.center(),
        ci.upper_bound(),
        ci.precision(),
    ]
}

fn extract_statistics_and_2cis_slice(
    statistic: f64,
    first_ci: &ConfidenceInterval,
    second_ci: &ConfidenceInterval,
) -> [f64; 9] {
    [
        statistic,
        first_ci.lower_bound(),
        first_ci.center(),
        first_ci.upper_bound(),
        first_ci.precision(),
        second_ci.lower_bound(),
        second_ci.center(),
        second_ci.upper_bound(),
        second_ci.precision(),
    ]
}
