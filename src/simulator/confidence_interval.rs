pub struct ConfidenceInterval {
    lower_bound: f64,
    upper_bound: f64,
    center: f64,
    precision: f64,
}

impl ConfidenceInterval {
    pub fn new(lower_bound: f64, upper_bound: f64) -> Self {
        Self {
            lower_bound,
            upper_bound,
            center: (lower_bound + upper_bound) / 2.0,
            precision: (upper_bound - lower_bound) / (upper_bound + lower_bound),
        }
    }

    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }

    pub fn center(&self) -> f64 {
        self.center
    }

    pub fn precision(&self) -> f64 {
        self.precision
    }

    pub fn check_convergence(first: Self, second: Self) -> bool {
        first.lower_bound < second.center
            && first.upper_bound > second.center
            && second.lower_bound < first.center
            && second.upper_bound > first.center
    }
}
