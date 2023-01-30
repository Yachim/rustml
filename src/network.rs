use crate::utils::functions::cost::CostFunc;
use crate::utils::math::dot_product;
use rand::Rng;

/// L = number of layers (except the first (input) layer)
/// l = current layer
/// N = number of neurons in the lth layer
/// n = current neuron from the layer N
/// M = number of neurons in the (l - 1)th layer
/// m = current neuron from the layer M
pub struct Network<'a> {
    /// training data
    /// vector of tuples - first element are inputs, second are the expected outputs
    training_data: Vec<(Vec<f32>, Vec<f32>)>,

    /// the input fields
    pub input: Vec<f32>,

    /// outer vector has len of L
    /// each element represents a layer (l)
    /// does not inluclude the first (input) layer
    ///
    /// inner vectors can have different len of N
    /// each element represents a neuron's (n) value before activation
    pub layers: Vec<Vec<f32>>,

    /// outer vector has len of L
    /// each element represents a layer (l)
    /// does not inluclude the first (input) layer
    ///
    /// inner vectors can have different len of N
    /// each element represents a neuron's (n) value after activation
    pub activated_layers: Vec<Vec<f32>>,

    /// same length as the last layer
    /// labels for the outputs
    out_labels: Vec<&'a str>,

    /// outer vector has len of (L - 1)
    /// each element represents all the weights between lth layer (N) and (l - 1)th layer (M)
    ///
    /// middle vectors can have different len of N
    /// each element represents all weights connected to neuron n
    ///
    /// inner vectors can have different len of M
    /// each element represents a weight conected from neuron m to neuron n
    pub weights: Vec<Vec<Vec<f32>>>,

    /// outer vector has len of L
    /// each element represents a layer (l)
    /// does not inluclude the first (input) layer
    ///
    /// inner vectors can have different len of N
    /// each element represents a bias of a neuron (n)
    biases: Vec<Vec<f32>>,

    /// vector has len of (L - 1)
    /// contains activation functions for all layers except the first (input) layer
    /// does not inluclude the first (input) layer
    activations: Vec<&'a dyn Fn(f32) -> f32>,
    pub activations_derivatives: Vec<&'a dyn Fn(f32) -> f32>,

    /// derivative of the cost function
    cost_func_derivative: CostFunc<'a>,

    /// learning rate of the network
    learning_rate: f32,

    batch_size: usize,

    /// number of iterations when training
    iteration_cnt: usize,
}

impl<'a> Network<'a> {
    /// Creates a new network
    ///  - training_data: vector of tuples: first element are inputs, second are the expected outputs,
    ///  - hidden_layers: vector of usize; each usize represents the number of neurons in a layer,
    ///  - out_labels: vector of strings; each string represents a label for the output,
    ///  - activations: activation functions for each layer,
    ///  - activations_derivatives: derivatives of activation functions for each layer,
    ///  - cost_func_derivative: cost functions,
    ///  - learning_rate: the integer to multiply the weights and biases,
    ///  - batch_size: size of batches in gradient descent,
    ///  - iteration_cnt: the number of iterations when training,
    pub fn new(
        training_data: Vec<(Vec<f32>, Vec<f32>)>,
        hidden_layers: Vec<usize>,
        out_labels: Vec<&'a str>,
        activations: Vec<&'a dyn Fn(f32) -> f32>,
        activations_derivatives: Vec<&'a dyn Fn(f32) -> f32>,
        cost_func_derivative: CostFunc<'a>,
        learning_rate: f32,
        batch_size: usize,
        iteration_cnt: usize,
    ) -> Self {
        assert_eq!(activations_derivatives.len(), activations.len());
        assert_eq!(activations.len(), hidden_layers.len() + 1);
        // all layers except the input (hidden + output)
        // contains neurons count
        let ls: Vec<usize> = hidden_layers
            .iter()
            .chain(&[out_labels.len()])
            .copied()
            .collect();

        let input = vec![0.0; training_data[0].0.len()];

        let mut layers: Vec<Vec<f32>> = vec![];
        let mut activated_layers: Vec<Vec<f32>> = vec![];

        for cnt in ls {
            layers.push(vec![0.0; cnt]);
            activated_layers.push(vec![0.0; cnt]);
        }

        let mut s = Self {
            training_data,
            input,
            layers,
            activated_layers,
            weights: vec![],
            biases: vec![],
            activations,
            activations_derivatives,
            cost_func_derivative,
            out_labels,
            learning_rate,
            batch_size,
            iteration_cnt,
        };

        s.initialize_params();

        s
    }

    /// randomly sets weights and biases
    fn initialize_params(&mut self) {
        let mut rng = rand::thread_rng();

        let mut biases: Vec<Vec<f32>> = vec![];
        let mut weights: Vec<Vec<Vec<f32>>> = vec![];

        for (i, layer) in self.layers.iter().enumerate() {
            let mut layer_biases: Vec<f32> = vec![];
            let mut layer_weights: Vec<Vec<f32>> = vec![];

            for _ in 0..layer.len() {
                layer_biases.push(rng.gen_range(0.0..=1.0));

                let mut neuron_weights: Vec<f32> = vec![];
                // count of neurons in prevous layer
                let neuron_cnt: usize;
                if i == 0 {
                    neuron_cnt = self.input.len();
                } else {
                    neuron_cnt = self.layers[i - 1].len();
                };

                for _ in 0..neuron_cnt {
                    neuron_weights.push(rng.gen_range(0.0..=1.0));
                }

                layer_weights.push(neuron_weights);
            }

            biases.push(layer_biases);
            weights.push(layer_weights);
        }

        self.weights = weights;
        self.biases = biases;
    }

    /// feedforwards (sets values) to the layer (in layers and activated_layers) with the given index
    fn feedforward_layer(&mut self, index: usize) {
        let prev_layer: Vec<f32> = if index == 0 {
            self.input.to_owned()
        } else {
            self.activated_layers[index - 1].to_owned()
        };

        for i in 0..self.layers[index].len() {
            self.layers[index][i] =
                dot_product(&prev_layer, &self.weights[index][i]) + self.biases[index][i];
            self.activated_layers[index][i] = (self.activations[index])(self.layers[index][i]);
        }
    }

    fn feedforward(&mut self) {
        for i in 0..self.layers.len() {
            self.feedforward_layer(i);
        }
    }

    fn gradient_descent(&mut self) {
        for batch_start_index in (0..self.training_data.len()).step_by(self.batch_size) {
            let batch = self.training_data[batch_start_index..batch_start_index + self.batch_size]
                .to_owned();

            let mut total_dws: Vec<Vec<Vec<f32>>> = vec![];
            let mut total_dbs: Vec<Vec<f32>> = vec![];

            for l in 0..self.weights.len() {
                let mut layer_dws: Vec<Vec<f32>> = vec![];
                let mut layer_dbs: Vec<f32> = vec![];

                for j in 0..self.weights[l].len() {
                    let mut neuron_dws: Vec<f32> = vec![];

                    for _ in 0..self.weights[l][j].len() {
                        neuron_dws.push(0.0);
                    }

                    layer_dws.push(neuron_dws);
                    layer_dbs.push(0.0);
                }

                total_dws.push(layer_dws);
                total_dbs.push(layer_dbs);
            }

            for (inputs, expected) in batch.iter() {
                self.input = inputs.to_vec();

                self.feedforward();

                let (dws, dbs) = (self.cost_func_derivative)(&self, expected.to_vec());

                for l in 0..dws.len() {
                    for j in 0..dws[l].len() {
                        for k in 0..dws[l][j].len() {
                            total_dws[l][j][k] += dws[l][j][k];
                            total_dbs[l][j] += dbs[l][j];
                        }
                    }
                }
            }

            for l in 0..total_dws.len() {
                for j in 0..total_dws[l].len() {
                    for k in 0..total_dws[l][j].len() {
                        self.weights[l][j][k] -=
                            total_dws[l][j][k] / (self.batch_size as f32) * self.learning_rate;
                    }

                    self.biases[l][j] -=
                        total_dbs[l][j] / (self.batch_size as f32) * self.learning_rate;
                }
            }
        }
    }

    pub fn train(&mut self) {
        for _ in 0..self.iteration_cnt {
            self.gradient_descent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Network;
    use crate::utils::functions::{
        activation::{sigmoid, sigmoid_deriv},
        cost::mse_deriv,
    };

    #[test]
    fn test_ws_cnt() {
        let net = Network::new(
            vec![], // no need for input batches
            3,
            vec![2, 3],
            vec!["1", "2"],
            vec![&sigmoid, &sigmoid, &sigmoid],
            vec![&sigmoid_deriv, &sigmoid_deriv, &sigmoid_deriv],
            &mse_deriv,
            0.5,
            0, // no need for batch_size
            1,
        );

        let mut total_ws = 0;
        for i in net.weights {
            for j in i {
                for _ in j {
                    total_ws += 1;
                }
            }
        }

        assert_eq!(total_ws, 18);
    }

    #[test]
    fn test_bias_cnt() {
        let net = Network::new(
            vec![], // no need for input batches
            3,
            vec![2, 3],
            vec!["1", "2"],
            vec![&sigmoid, &sigmoid, &sigmoid],
            vec![&sigmoid_deriv, &sigmoid_deriv, &sigmoid_deriv],
            &mse_deriv,
            0.5,
            0, // no need for batch_size
            1,
        );

        let mut total_biases = 0;
        for i in net.biases {
            for _ in i {
                total_biases += 1;
            }
        }

        assert_eq!(total_biases, 7);
    }

    #[test]
    fn test_neurons_cnt() {
        let net = Network::new(
            vec![], // no need for input batches
            3,
            vec![2, 3],
            vec!["1", "2"],
            vec![&sigmoid, &sigmoid, &sigmoid],
            vec![&sigmoid_deriv, &sigmoid_deriv, &sigmoid_deriv],
            &mse_deriv,
            0.5,
            0, // no need for batch_size
            1,
        );

        let mut total_neurons = 0;
        for i in net.layers {
            for _ in i {
                total_neurons += 1;
            }
        }
        total_neurons += net.input.len();

        assert_eq!(total_neurons, 10);
    }

    #[test]
    fn test_feedforward_layer() {
        let mut net = Network::new(
            vec![], // no need for input batches
            3,
            vec![],
            vec!["1", "2", "3"],
            vec![&sigmoid],
            vec![&sigmoid_deriv],
            &mse_deriv,
            0.5,
            0, // no need for batch_size
            1,
        );

        net.input = vec![3.0, 2.0];
        net.weights[0][0] = vec![3.0, 4.0];
        net.weights[0][1] = vec![2.0, 4.0];
        net.weights[0][2] = vec![3.0, 5.0];
        net.biases[0] = vec![1.0, 1.0, 3.0];

        net.feedforward_layer(0);

        assert_eq!(net.layers[0][0], 18.0);
        assert_eq!(net.layers[0][1], 15.0);
        assert_eq!(net.layers[0][2], 22.0);
    }
}
