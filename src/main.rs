// Módulo onde definimos o simulador
mod simulator;

// Importamos nosso simulador e o enum de política de fila
use simulator::simulator;
use simulator::QueuePolicy;

fn main() {
    // Pequenos rhos, para constatar a corretude do simulador
    simulator(0.1, 10_000, 3200, QueuePolicy::FCFS);
    simulator(0.1, 10_000, 3200, QueuePolicy::LCFS);

    simulator(0.01, 10_000, 3200, QueuePolicy::FCFS);
    simulator(0.01, 10_000, 3200, QueuePolicy::LCFS);

    simulator(0.001, 10_000, 3200, QueuePolicy::FCFS);
    simulator(0.001, 10_000, 3200, QueuePolicy::LCFS);

    // Para 0.0001 começamos com 31000 pois menos do que isso ele não convergia
    simulator(0.000_1, 31_000, 3200, QueuePolicy::FCFS);
    simulator(0.000_1, 31_000, 3200, QueuePolicy::LCFS);

    // Simulação com os rhos pedidos
    let rhos = [0.2, 0.4, 0.6, 0.8];
    for &rho in &rhos {
        simulator(rho, 1_000, 3200, QueuePolicy::FCFS);
        simulator(rho, 1_000, 3200, QueuePolicy::LCFS);
    }
    // Fizemos rho = 0.9 começar em 15000 pois demorava bem mais para convergir
    simulator(0.9, 15_000, 3200, QueuePolicy::FCFS);
    simulator(0.9, 15_000, 3200, QueuePolicy::LCFS);
}
