use std::iter::zip;

pub struct CostFunc<'a> {
    pub function: &'a dyn Fn(&Vec<f32>, &Vec<f32>) -> f32,
    pub derivative: &'a dyn Fn(f32, f32) -> f32,

    pub description: &'a str,

    /// latex formula
    pub formula: &'a str,
    /// latex formula
    pub formula_derivative: &'a str,
}

/// Mean Squared Error
///  - network: the Network instance to apply on
///  - expected: expected output values; same length as last layer of network
///
/// returns the cost
fn mse(predicted: &Vec<f32>, expected: &Vec<f32>) -> f32 {
    assert_eq!(predicted.len(), expected.len());

    let sum: f32 = zip(predicted, expected)
        .map(|(a, y)| (a - y).powf(2.0))
        .sum();

    sum / (expected.len() as f32)
}

/// Derivative of the Mean Squared Error with respect to the activations (predictions)
///  - network: the Network instance to apply on
///  - expected: expected output values; same length as last layer of network
///
/// returns the derivatives of the cost function with respect to activations in the last layer
fn mse_deriv(predicted: f32, expected: f32) -> f32 {
    2.0 * (predicted - expected)
}

pub const MSE: CostFunc = CostFunc {
    function: &mse,
    derivative: &mse_deriv,

    description: "",

    formula: "",
    formula_derivative: "",
};

#[cfg(test)]
mod tests {
    use super::mse;

    #[test]
    fn test_mse() {
        assert_eq!(mse(&vec![1.0], &vec![1.0]), 0.0);
        assert_eq!(mse(&vec![1.0], &vec![0.5]), 0.25);
        assert_eq!(mse(&vec![1.0], &vec![0.0]), 1.0);
    }
}
