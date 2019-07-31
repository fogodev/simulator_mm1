// Módulo onde definimos o simulador
mod simulator;

// Importamos nosso simulador e o enum de política de fila
use simulator::simulator;
use simulator::QueuePolicy;
use crate::simulator::QueueMode;

fn main() {

    // A taxa rho é desconsiderada quando executamos o simulador em modo de verificar corretude do mesmo
    simulator(999.0, 1_000, 3200, QueuePolicy::FCFS, QueueMode::CheckCorrectness);
    simulator(999.0, 1_000, 3200, QueuePolicy::LCFS, QueueMode::CheckCorrectness);

    // Pequenos rhos, para constatar a corretude do simulador
    simulator(0.1, 10_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
    simulator(0.1, 10_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);

    simulator(0.01, 10_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
    simulator(0.01, 10_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);

    simulator(0.001, 10_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
    simulator(0.001, 10_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);

    // Para 0.0001 começamos com 31000 pois menos do que isso ele não convergia
    simulator(0.000_1, 31_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
    simulator(0.000_1, 31_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);

    // Simulação com os rhos pedidos
    let rhos = [0.2, 0.4, 0.6, 0.8];
    for &rho in &rhos {
        simulator(rho, 1_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
        simulator(rho, 1_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);
    }
    // Fizemos rho = 0.9 começar em 15000 pois demorava bem mais para convergir
    simulator(0.9, 15_000, 3200, QueuePolicy::FCFS, QueueMode::ForReal);
    simulator(0.9, 15_000, 3200, QueuePolicy::LCFS, QueueMode::ForReal);


}
