use crate::simulator::confidence_interval::ConfidenceInterval;

// Percentis da Distribuição Chi² para uma confiança de 95% e 3199 graus de liberdade.
// Obtido através da biblioteca scipy no Python
const CHI_SQUARE_LOWER_PERCENTILE: f64 = 3_044.130_201_770_939_5;
const CHI_SQUARE_UPPER_PERCENTILE: f64 = 3_357.658_239_649_767_4;

// Percentil da Distribuição T-Student para uma confiança de 95% e 3199 de liberdade.
// Obtido através da biblioteca scipy no Python
const T_STUDENT_PERCENTILE: f64 = 1.960_705_826_924_122_4;

pub struct Sample {
    values: Vec<f64>,
}

impl Sample {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn append(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> f64 {
        if !self.values.is_empty() {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        } else {
            0.0
        }
    }

    pub fn variance(&self) -> f64 {
        let mean = self.mean();
        if mean == 0.0 {
            0.0
        } else {
            self.values
                .iter()
                .fold(0.0, |sum, &value| sum + (value - mean).powi(2))
                / (self.values.len() - 1) as f64
        }
    }

    pub fn t_student_95percent(&self) -> ConfidenceInterval {
        let mean = self.mean();
        let t_student_times_sqrt_of_variance_by_sample_count =
            T_STUDENT_PERCENTILE * (self.variance() / self.values.len() as f64).sqrt();
        ConfidenceInterval::new(
            mean - t_student_times_sqrt_of_variance_by_sample_count,
            mean + t_student_times_sqrt_of_variance_by_sample_count,
        )
    }

    pub fn chi_square_95percent(&self, sample_variance: f64) -> ConfidenceInterval {
        let n_minus_one_times_variance = (self.values.len() - 1) as f64 * sample_variance;
        ConfidenceInterval::new(
            n_minus_one_times_variance / CHI_SQUARE_UPPER_PERCENTILE,
            n_minus_one_times_variance / CHI_SQUARE_LOWER_PERCENTILE,
        )
    }
}
