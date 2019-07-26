mod simulator;

use std::time::Instant;

use simulator::simulator;
use simulator::QueuePolicy;

pub fn timing(rho: f64, transient_phase: usize, round_size: usize, queue_policy: QueuePolicy) {
    let now = Instant::now();
    simulator(rho, transient_phase, round_size, 3200, queue_policy, 9999);
    println!(
        "ρ = {}; Tempo decorrido = {:0.5}s\n\n",
        rho,
        now.elapsed().as_millis() as f64 / 1000.0
    );
}

fn main() {
    let runs = [1_000, 5_000, 10_000, 15_000, 20_000];

    for &transient_phase in &runs {
        for &round_size in &runs {
            timing(0.2, transient_phase, round_size, QueuePolicy::FCFS);
            timing(0.2, transient_phase, round_size, QueuePolicy::LCFS);

            timing(0.4, transient_phase, round_size, QueuePolicy::FCFS);
            timing(0.4, transient_phase, round_size, QueuePolicy::LCFS);

            timing(0.6, transient_phase, round_size, QueuePolicy::FCFS);
            timing(0.6, transient_phase, round_size, QueuePolicy::LCFS);

            timing(0.8, transient_phase, round_size, QueuePolicy::FCFS);
            timing(0.8, transient_phase, round_size, QueuePolicy::LCFS);

            timing(0.9, transient_phase, round_size, QueuePolicy::FCFS);
            timing(0.9, transient_phase, round_size, QueuePolicy::LCFS);
        }
    }
}
