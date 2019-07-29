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

    // Calcula uma amostra exponencial a partir de um número aleatório entre (0, 1) gerado
    pub fn get(&mut self, lambda: f64) -> f64 {
        // Usando o menor float positivo para garantir que não teremos ln(0)
        -self
            .random_number_generator
            .gen_range(std::f64::MIN_POSITIVE, 1.0f64)
            .ln()
            / lambda
    }
}
