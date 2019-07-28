// Módulo onde definimos o simulador
mod simulator;

// Funcionalidade de temporização da biblioteca padrão
use std::time::Instant;

// Importamos nosso simulador e o enum de política de fila
use simulator::simulator;
use simulator::QueuePolicy;

// Função simplificada para chamar o simulador e medir seu tempo de execução
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
    // Pequenos rhos, para constatar a corretude do simulador
    let smaller_rhos = [0.1, 0.01, 0.001, 0.0001];

    for &small_rho in &smaller_rhos {
        timing(small_rho, 10_000, 20_000, QueuePolicy::FCFS);
        timing(small_rho, 10_000, 20_000, QueuePolicy::LCFS);
    }

    // Tamanhos de fase transiente e quantidade de fregueses por rodada que queremos simular
    let runs = [1_000, 5_000, 10_000, 15_000, 20_000];

    // Rodando o simulador para todas as combinação de fase transiente e fregueses por rodada
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
