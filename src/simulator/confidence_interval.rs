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
        first.lower_bound <= second.center
            && first.upper_bound >= second.center
            && second.lower_bound <= first.center
            && second.upper_bound >= first.center
    }

    pub fn value_is_inside(&self, value: f64) -> bool {
        // Verificamos se o valor passado está dentro dos limites inferior e superior
        // ou então bastante próximo desses limites, devido a possíveis erros numéricos de float
        (self.lower_bound <= value
            || 1.0 - f64::min(self.lower_bound, value) / f64::max(self.lower_bound, value) <= 0.01)
            && (value <= self.upper_bound
                || 1.0 - f64::min(self.upper_bound, value) / f64::max(self.upper_bound, value)
                    <= 0.01)
    }
}
