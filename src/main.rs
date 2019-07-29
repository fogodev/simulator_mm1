// Módulo onde definimos o simulador
mod simulator;

// Funcionalidade de temporização da biblioteca padrão
use std::time::SystemTime;

// Importamos nosso simulador e o enum de política de fila
use simulator::simulator;
use simulator::QueuePolicy;

fn main() {
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter o tempo do sistema")
        .as_secs();

    // Pequenos rhos, para constatar a corretude do simulador
    let smaller_rhos = [0.1, 0.01, 0.001, 0.000_1];
    for &small_rho in &smaller_rhos {
        simulator(small_rho, 20_000, 3200, QueuePolicy::FCFS, seed);
        simulator(small_rho, 20_000, 3200, QueuePolicy::LCFS, seed);
    }

    // Simulação com os rhos pedidos
    let rhos = [0.2, 0.4, 0.6, 0.8, 0.9];
    for &rho in &rhos {
        simulator(rho, 10_000, 3200, QueuePolicy::FCFS, seed);
        simulator(rho, 10_000, 3200, QueuePolicy::LCFS, seed);
    }
}
