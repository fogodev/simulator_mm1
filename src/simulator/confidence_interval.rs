// Struct para representar um intervalo de confiança
pub struct ConfidenceInterval {
    lower_bound: f64,
    upper_bound: f64,
    center: f64,
    precision: f64,
}

impl ConfidenceInterval {
    // Instancia um novo IC a partir dos limites inferior e superior e calcula centro e precisão
    pub fn new(lower_bound: f64, upper_bound: f64) -> Self {
        Self {
            lower_bound,
            upper_bound,
            center: (lower_bound + upper_bound) / 2.0,
            precision: (upper_bound - lower_bound) / (upper_bound + lower_bound),
        }
    }

    // Getter do tempo do limite inferior
    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }
    // Getter do tempo do limite superior
    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }
    // Getter do tempo do centro
    pub fn center(&self) -> f64 {
        self.center
    }
    // Getter do tempo da precisão
    pub fn precision(&self) -> f64 {
        self.precision
    }
    // Verifica a convergência entre dois intervalos de confiança
    pub fn check_convergence(first: Self, second: Self) -> bool {
        first.lower_bound < second.center
            && first.upper_bound > second.center
            && second.lower_bound < first.center
            && second.upper_bound > first.center
    }
}
