// Struct para acumular os valores e tempos de um processo estocástico
pub struct StochasticProcessSample {
    arrivals_times: Vec<f64>,
    values: Vec<usize>,
}

impl StochasticProcessSample {
    // Instancia um novo objeto, já tendo uma ideia aproximada de quantos valores serão recebidos
    pub fn new(capacity: usize) -> Self {
        Self {
            arrivals_times: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    // Adiciona novos valores e tempos nos nossos vetores
    pub fn append(&mut self, time: f64, value: usize) {
        self.arrivals_times.push(time);
        self.values.push(value);
    }

    // Calcula o estimador da média
    pub fn mean(&self) -> f64 {
        let sample_count = self.values.len();
        if sample_count > 0 {
            let mut result = 0.0;
            for index in 0..(sample_count - 1) {
                result += self.values[index] as f64
                    * (self.arrivals_times[index + 1] - self.arrivals_times[index])
            }
            result / (self.arrivals_times[sample_count - 1] - self.arrivals_times[0])
        } else {
            0.0
        }
    }

    // Calcula o estimador da variância
    pub fn variance(&self) -> f64 {
        let sample_count = self.values.len();
        if sample_count > 0 {
            let mut second_moment = 0.0;
            for index in 0..(sample_count - 1) {
                second_moment += self.values[index].pow(2) as f64
                    * (self.arrivals_times[index + 1] - self.arrivals_times[index])
            }
            second_moment /= self.arrivals_times[sample_count - 1] - self.arrivals_times[0];
            second_moment - self.mean() * self.mean()
        } else {
            0.0
        }
    }
}
