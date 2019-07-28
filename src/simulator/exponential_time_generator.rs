// Importando os elementos da biblioteca de números aleatórios
use rand::prelude::*;

// Struct para armazenar o objeto gerador de números aleatórios
pub struct ExponentialTime {
    random_number_generator: StdRng,
}

impl ExponentialTime {
    // Instancia um novo gerador de amostras exponenciais
    pub fn new(seed: u64) -> Self {
        Self {
            random_number_generator: StdRng::seed_from_u64(seed),
        }
    }

    // Calcula uma amostra exponencial a partir de um número aleatório entre [0, 1) gerado
    pub fn get(&mut self, lambda: f64) -> f64 {
        -1.0 * (1.0 / lambda) * (self.random_number_generator.gen_range(0.0f64, 1.0f64)).ln()
    }
}
