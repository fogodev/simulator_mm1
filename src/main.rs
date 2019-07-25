mod simulator;

use std::time::Instant;

use simulator::simulator;
use simulator::QueuePolicy;

pub fn timing_simulation(rho: f64, queue_policy: QueuePolicy) {
    let now = Instant::now();
    simulator(rho, 10_000, 5_000, 3200, queue_policy, 9999);
    println!(
        "ρ = {}; Tempo decorrido = {:0.5}s\n\n",
        rho,
        now.elapsed().as_millis() as f64 / 1000.0
    );
}

fn main() {
    timing_simulation(0.2, QueuePolicy::FCFS);
    timing_simulation(0.2, QueuePolicy::LCFS);

    timing_simulation(0.4, QueuePolicy::FCFS);
    timing_simulation(0.4, QueuePolicy::LCFS);

    timing_simulation(0.6, QueuePolicy::FCFS);
    timing_simulation(0.6, QueuePolicy::LCFS);

    timing_simulation(0.8, QueuePolicy::FCFS);
    timing_simulation(0.8, QueuePolicy::LCFS);

    timing_simulation(0.9, QueuePolicy::FCFS);
    timing_simulation(0.9, QueuePolicy::LCFS);
}
