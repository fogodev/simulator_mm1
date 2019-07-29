// Estrutura de dados HashMap da biblioteca padrão
use std::collections::HashMap;

// Struct que representa um freguês na fila, seu tempo de atendimento e seus possíveis eventos
pub struct Client {
    x: f64,
    start_event: HashMap<String, f64>,
    end_event: HashMap<String, f64>,
    color: usize,
}

impl Client {
    // Instancia um novo freguês, com o tempo de atendimento e HashMaps de eventos
    pub fn new(x: f64, color: usize) -> Self {
        Self {
            x,
            start_event: HashMap::new(),
            end_event: HashMap::new(),
            color,
        }
    }

    // Getter do tempo de atendimento
    pub fn x(&self) -> f64 {
        self.x
    }

    // Getter da cor desse cliente
    pub fn color(&self) -> usize {
        self.color
    }

    // Registra o começo de um novo evento, tal como entrada em atendimento ou na fila de espera
    pub fn register_start(&mut self, name: &str, arrival_time: f64) {
        self.start_event.insert(name.to_string(), arrival_time);
    }

    // Registra o fim de um evento, tal como saída de atendimento ou da fila de espera
    pub fn register_end(&mut self, name: &str, leave_time: f64) {
        self.end_event.insert(name.to_string(), leave_time);
    }

    // Calcula o tempo total de algum evento
    pub fn calculate_event_time(&self, name: &str) -> f64 {
        assert!(
            self.start_event.contains_key(name) && self.end_event.contains_key(name),
            format!(
                "O evento {} deve ter sido iniciado e encerrado para calcular seu tempo",
                name
            )
        );
        self.end_event[name] - self.start_event[name]
    }
}
