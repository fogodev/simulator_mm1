use std::collections::HashMap;

pub struct Client {
    x: f64,
    start_event: HashMap<String, f64>,
    end_event: HashMap<String, f64>,
}

impl Client {
    pub fn new(x: f64) -> Self {
        Self {
            x,
            start_event: HashMap::new(),
            end_event: HashMap::new(),
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn register_start(&mut self, name: &str, arrival_time: f64) {
        self.start_event.insert(name.to_string(), arrival_time);
    }

    pub fn register_end(&mut self, name: &str, leave_time: f64) {
        self.end_event.insert(name.to_string(), leave_time);
    }

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
