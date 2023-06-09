// TODO: fix test failures and implement test for softmax backprop
pub mod classification;
use chrono::offset::Local;
use rand::{seq::SliceRandom, thread_rng};

use super::{
    Debuggable, LayerNeurons, LayerWeights, NetworkNeurons, NetworkWeights, NeuronWeights,
    Predictable, Resetable, Shape, Trainable,
};
use crate::{
    functions::{
        activation::{ActivationFunc, FuncElementWiseDependency},
        cost::CostFunc,
        input_normalizations::NormalizationFunc,
    },
    utils::math::{
        add_2d_vecs, add_3d_vecs, divide_vector_float_2d, divide_vector_float_3d, dot_product,
        multiply_vector_float_2d, multiply_vector_float_3d, subtract_2d_vecs, subtract_3d_vecs,
    },
};

struct MultiLayerPerceptron<'a> {
    shape: Shape,
    activation_funcs: Vec<&'a ActivationFunc<'a>>,
    cost_func: &'a CostFunc<'a>,
    normalization_func: &'a NormalizationFunc<'a>,

    inputs: LayerNeurons,
    normalized_inputs: LayerNeurons,
    weights: NetworkWeights,
    biases: NetworkNeurons,
    layers: NetworkNeurons,
    activated_layers: NetworkNeurons,
}

impl<'a> MultiLayerPerceptron<'a> {
    fn normalize_input(&mut self) {
        let normalization_func = self.normalization_func.func;

        let inputs = &self.inputs;
        let normalized_inputs = normalization_func(inputs);
        self.normalized_inputs = normalized_inputs;
    }

    fn activate_independent_layer(&mut self, layer_i: usize) {
        let layer_activation_func = match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Independent { func, .. } => func,
            FuncElementWiseDependency::Dependent { .. } => unreachable!(),
        };

        let layer = &self.layers[layer_i];

        for (neuron_i, &neuron) in layer.iter().enumerate() {
            self.activated_layers[layer_i][neuron_i] = layer_activation_func(neuron);
        }
    }

    fn activate_dependent_layer(&mut self, layer_i: usize) {
        let layer_activation_func = match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Dependent { func, .. } => func,
            FuncElementWiseDependency::Independent { .. } => unreachable!(),
        };

        let layer = &self.layers[layer_i];

        self.activated_layers[layer_i] = layer_activation_func(&layer);
    }

    fn feedforward_layer(&mut self, layer_i: usize) {
        let prev_layer = if layer_i == 0 {
            &self.normalized_inputs
        } else {
            &self.activated_layers[layer_i - 1]
        };
        let layer_weights = &self.weights[layer_i];
        let layer_biases = &self.biases[layer_i];

        let mut new_layer: LayerNeurons = vec![];
        for neuron_i in 0..layer_weights.len() {
            let neuron_weights = &layer_weights[neuron_i];
            let neuron_bias = layer_biases[neuron_i];

            let new_neuron = dot_product(prev_layer, neuron_weights) + neuron_bias;
            new_layer.push(new_neuron);
        }

        self.layers[layer_i] = new_layer;

        match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Independent { .. } => {
                self.activate_independent_layer(layer_i);
            }
            FuncElementWiseDependency::Dependent { .. } => {
                self.activate_dependent_layer(layer_i);
            }
        }
    }

    fn feedforward(&mut self) {
        let layer_cnt = self.layers.len();

        self.normalize_input();

        for layer_i in 0..layer_cnt {
            self.feedforward_layer(layer_i);
        }
    }

    /// calculates dC/dz[l]
    fn calculate_cost_value_derivative_independent(
        &self,
        layer_i: usize,
        prev_cost_value_derivatives: Option<&LayerNeurons>,
        expected: Option<&LayerNeurons>,
    ) -> LayerNeurons {
        let activation_func_derivative = match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Independent { derivative, .. } => derivative,
            FuncElementWiseDependency::Dependent { .. } => unreachable!(),
        };

        // z[l]
        let layer = &self.layers[layer_i];
        // a[l]
        let activated_layer = &self.activated_layers[layer_i];
        // w[l + 1]
        let prev_weights = &self.weights[layer_i + 1];

        // dC/da[l]
        let cost_activation_derivatives: Vec<f32> = if layer_i == self.layers.len() - 1 {
            let expected = expected.unwrap();

            let cost_func_derivative = self.cost_func.derivative;

            activated_layer
                .iter()
                .enumerate()
                .map(|(j, &activated_neuron)| cost_func_derivative(activated_neuron, expected[j]))
                .collect()
        } else {
            (0..activated_layer.len())
                .map(|j| {
                    prev_cost_value_derivatives
                        .unwrap()
                        .iter()
                        .enumerate()
                        .fold(0.0, |acc, (i, &dc_dzi)| acc + dc_dzi * prev_weights[i][j])
                })
                .collect()
        };

        // da[l]/dz[l]
        let activation_value_derivatives = layer.iter().map(|&z| activation_func_derivative(z));

        // dC/dz[l]
        let cost_value_derivatives: LayerNeurons = cost_activation_derivatives
            .iter()
            .zip(activation_value_derivatives)
            .map(|(&dc_da, da_dz)| dc_da * da_dz)
            .collect();

        cost_value_derivatives
    }

    /// calculates dC/dz[l]
    fn calculate_cost_value_derivative_dependent(
        &self,
        layer_i: usize,
        prev_cost_value_derivatives: Option<&LayerNeurons>,
        expected: Option<&LayerNeurons>,
    ) -> LayerNeurons {
        let activation_func_derivative = match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Dependent { derivative, .. } => derivative,
            FuncElementWiseDependency::Independent { .. } => unreachable!(),
        };

        // z[l]
        let layer = &self.layers[layer_i];
        // a[l]
        let activated_layer = &self.activated_layers[layer_i];
        // w[l + 1]
        let prev_weights = &self.weights[layer_i + 1];

        // dC/da[l]
        let cost_activation_derivatives: Vec<f32> = if layer_i == self.layers.len() - 1 {
            let expected = expected.unwrap();

            let cost_func_derivative = self.cost_func.derivative;

            activated_layer
                .iter()
                .enumerate()
                .map(|(j, &activated_neuron)| cost_func_derivative(activated_neuron, expected[j]))
                .collect()
        } else {
            (0..activated_layer.len())
                .map(|j| {
                    prev_cost_value_derivatives
                        .unwrap()
                        .iter()
                        .enumerate()
                        .fold(0.0, |acc, (i, &dc_dzi)| acc + dc_dzi * prev_weights[i][j])
                })
                .collect()
        };

        // da[l]/dz[l]
        let activation_jacobian = activation_func_derivative(layer);

        // dC/dz[l]
        let cost_value_derivatives: LayerNeurons = (0..activation_jacobian.len())
            .map(|j| {
                cost_activation_derivatives
                    .iter()
                    .enumerate()
                    .fold(0.0, |acc, (i, &dc_da)| {
                        acc + dc_da * activation_jacobian[i][j]
                    })
            })
            .collect();

        cost_value_derivatives
    }

    /// calculates dC/dz[l]
    fn calculate_cost_value_derivative(
        &self,
        layer_i: usize,
        prev_cost_value_derivatives: Option<&LayerNeurons>,
        expected: Option<&LayerNeurons>,
    ) -> LayerNeurons {
        match &self.activation_funcs[layer_i].funcs {
            FuncElementWiseDependency::Dependent { .. } => self
                .calculate_cost_value_derivative_dependent(
                    layer_i,
                    prev_cost_value_derivatives,
                    expected,
                ),
            FuncElementWiseDependency::Independent { .. } => self
                .calculate_cost_value_derivative_independent(
                    layer_i,
                    prev_cost_value_derivatives,
                    expected,
                ),
        }
    }

    /// returns derivatives in order: dC/dw[l], dC/db[l], dC/dz[l]
    ///
    /// prev_cost_activation_derivatives: dC/da[l + 1]
    /// not needed only when computing last layer
    ///
    /// expected: only needed when computing last_layer
    ///
    /// prev as in previous iteration (next) since the iteration should be backwards
    fn backprop_layer(
        &self,
        layer_i: usize,
        prev_cost_value_derivatives: Option<&LayerNeurons>,
        expected: Option<&LayerNeurons>,
    ) -> (LayerWeights, LayerNeurons, LayerNeurons) {
        // a[l - 1]
        let next_activated_layer = if layer_i == 0 {
            &self.normalized_inputs
        } else {
            &self.activated_layers[layer_i - 1]
        };

        // dC/dz[l]
        let cost_value_derivatives =
            self.calculate_cost_value_derivative(layer_i, prev_cost_value_derivatives, expected);

        // dC/dw[l]
        let cost_weight_derivatives: LayerWeights = cost_value_derivatives
            .iter()
            .map(|dc_dzj| {
                next_activated_layer
                    .iter()
                    .map(|next_ak| dc_dzj * next_ak)
                    .collect()
            })
            .collect();

        // dC/db[l]
        let cost_bias_derivatives = cost_value_derivatives.clone();

        (
            cost_weight_derivatives,
            cost_bias_derivatives,
            cost_value_derivatives,
        )
    }

    fn backprop(&self, expected: &LayerNeurons) -> (NetworkWeights, NetworkNeurons) {
        let (last_layer_dws, last_layer_dbs, last_layer_dzs) =
            self.backprop_layer(self.layers.len() - 1, None, Some(expected));

        let mut dws: NetworkWeights = vec![last_layer_dws];
        let mut dbs: NetworkNeurons = vec![last_layer_dbs];
        let mut layer_dzs: LayerNeurons = last_layer_dzs;

        for layer_i in (0..self.layers.len() - 1).rev() {
            let (curr_layer_dws, curr_layer_dbs, curr_layer_dzs) =
                self.backprop_layer(layer_i, Some(&layer_dzs), None);

            dws.insert(0, curr_layer_dws);
            dbs.insert(0, curr_layer_dbs);
            layer_dzs = curr_layer_dzs;
        }

        (dws, dbs)
    }

    fn update_weights_and_biases(
        &mut self,
        dws: NetworkWeights,
        dbs: NetworkNeurons,
        learning_rate: f32,
    ) {
        let w_diff = subtract_3d_vecs(&self.weights, &dws);
        let b_diff = subtract_2d_vecs(&self.biases, &dbs);

        self.weights = multiply_vector_float_3d(&w_diff, learning_rate);
        self.biases = multiply_vector_float_2d(&b_diff, learning_rate);
    }

    fn batch_gradient_descent(
        &mut self,
        batch: &Vec<(LayerNeurons, LayerNeurons)>,
        batch_size: usize,
        learning_rate: f32,
    ) {
        for batch_start in (0..batch.len()).step_by(batch_size) {
            let mini_batch = if batch.len() - batch_start >= batch_size {
                &batch[batch_start..batch_start + batch_size]
            } else {
                &batch[batch_start..]
            };
            let mut avg_dws: NetworkWeights = vec![];
            let mut avg_dbs: NetworkNeurons = vec![];

            for data in mini_batch {
                let inputs = &data.0;
                let expected = &data.1;

                self.predict(inputs);
                let (dws, dbs) = self.backprop(&expected);

                if avg_dws.len() == 0 && avg_dbs.len() == 0 {
                    avg_dws = dws;
                    avg_dbs = dbs;
                } else {
                    avg_dws = add_3d_vecs(&avg_dws, &dws);
                    avg_dbs = add_2d_vecs(&avg_dbs, &dbs);
                }
            }

            avg_dws = divide_vector_float_3d(&avg_dws, mini_batch.len() as f32);
            avg_dbs = divide_vector_float_2d(&avg_dbs, mini_batch.len() as f32);
            self.update_weights_and_biases(avg_dws, avg_dbs, learning_rate);
        }
    }

    fn new(
        shape: Shape,
        activation_funcs: Vec<&'a ActivationFunc>,
        cost_func: &'a CostFunc,
        normalization_func: &'a NormalizationFunc,
    ) -> Self {
        assert_eq!(activation_funcs.len(), shape.len() - 1);
        let mut net = Self {
            shape,
            activation_funcs,
            cost_func,
            normalization_func,

            inputs: vec![],
            normalized_inputs: vec![],
            weights: vec![],
            biases: vec![],
            layers: vec![],
            activated_layers: vec![],
        };

        net.reset_params();
        net
    }
}

impl Resetable for MultiLayerPerceptron<'_> {
    fn reset_params(&mut self) {
        let shape = &self.shape;

        let mut new_weights: NetworkWeights = vec![];
        let mut new_biases: NetworkNeurons = vec![];
        let mut new_layers: NetworkNeurons = vec![];
        let mut new_activated_layers: NetworkNeurons = vec![];

        let new_inputs = vec![0.0; shape[0]];
        let new_normalized_inputs = vec![0.0; shape[0]];

        let weights_init_funcs = &self.activation_funcs;

        for (layer_index, &layer_neuron_cnt) in shape.iter().enumerate().skip(1) {
            let layer: LayerNeurons = vec![0.0; layer_neuron_cnt];
            new_layers.push(layer);

            let activated_layer: LayerNeurons = vec![0.0; layer_neuron_cnt];
            new_activated_layers.push(activated_layer);

            let layer_biases: Vec<f32> = vec![0.0; layer_neuron_cnt];
            new_biases.push(layer_biases);

            let mut layer_weights: LayerWeights = vec![];

            let prev_layer_neuron_cnt = shape[layer_index - 1];

            let layer_weights_init_func = weights_init_funcs[layer_index - 1].init_func.func;

            for _ in 0..layer_neuron_cnt {
                let mut neuron_weights: NeuronWeights = vec![];

                for _ in 0..prev_layer_neuron_cnt {
                    neuron_weights.push(layer_weights_init_func(prev_layer_neuron_cnt));
                }

                layer_weights.push(neuron_weights);
            }

            new_weights.push(layer_weights);
        }

        self.weights = new_weights;
        self.biases = new_biases;
        self.layers = new_layers;
        self.activated_layers = new_activated_layers;

        self.inputs = new_inputs;
        self.normalized_inputs = new_normalized_inputs;
    }
}

impl Predictable for MultiLayerPerceptron<'_> {
    fn get_highest_output(&self) -> (f32, usize) {
        let (i, val) = self
            .activated_layers
            .last()
            .unwrap()
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap();

        (*val, i)
    }

    fn predict(&mut self, inputs: &LayerNeurons) {
        self.inputs = inputs.to_vec();
        self.feedforward();
    }
}

impl Trainable for MultiLayerPerceptron<'_> {
    fn train(
        &mut self,
        iteration_cnt: usize,
        batch: &Vec<(LayerNeurons, LayerNeurons)>,
        batch_size: usize,
        learning_rate: f32,
    ) {
        let time_start = Local::now();
        println!("beginning training at {time_start}");

        let mut shuffled_batch = batch.clone();

        for i in 0..iteration_cnt {
            let epoch = i + 1;
            let time_epoch = Local::now();

            println!("beginning training epoch {epoch} out of {iteration_cnt} at {time_epoch}");

            shuffled_batch.shuffle(&mut thread_rng());
            self.batch_gradient_descent(&shuffled_batch, batch_size, learning_rate);
        }

        let time_end = Local::now();
        println!("finishing training at {time_end}");
    }
}

impl Debuggable for MultiLayerPerceptron<'_> {
    fn get_weights(&self) -> &NetworkWeights {
        &self.weights
    }

    fn get_biases(&self) -> &NetworkNeurons {
        &self.biases
    }

    fn get_inputs(&self) -> &LayerNeurons {
        &self.inputs
    }

    fn get_normalized_inputs(&self) -> &LayerNeurons {
        &self.normalized_inputs
    }

    fn get_layers(&self) -> &NetworkNeurons {
        &self.layers
    }

    fn get_activated_layers(&self) -> &NetworkNeurons {
        &self.activated_layers
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiLayerPerceptron, Predictable};
    use crate::functions::{
        activation::{RELU, SIGMOID, SOFTMAX},
        cost::MSE,
        input_normalizations::NO_NORMALIZATION,
    };

    #[test]
    fn test_ws_cnt() {
        let net = MultiLayerPerceptron::new(
            vec![3, 2, 3, 2],
            vec![&SIGMOID, &SIGMOID, &SIGMOID],
            &MSE,
            &NO_NORMALIZATION,
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
        let net = MultiLayerPerceptron::new(
            vec![3, 2, 3, 2],
            vec![&SIGMOID, &SIGMOID, &SIGMOID],
            &MSE,
            &NO_NORMALIZATION,
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
        let net = MultiLayerPerceptron::new(
            vec![3, 2, 3, 2],
            vec![&SIGMOID, &SIGMOID, &SIGMOID],
            &MSE,
            &NO_NORMALIZATION,
        );

        let mut total_neurons = 0;
        for i in net.layers {
            for _ in i {
                total_neurons += 1;
            }
        }
        total_neurons += net.inputs.len();

        assert_eq!(total_neurons, 10);
    }

    #[test]
    fn test_feedforward_layer1() {
        let mut net =
            MultiLayerPerceptron::new(vec![2, 3], vec![&SIGMOID], &MSE, &NO_NORMALIZATION);
        // test initialization
        assert_eq!(net.inputs.len(), 2);
        assert_eq!(net.normalized_inputs.len(), 2);

        assert_eq!(net.layers.len(), 1);
        assert_eq!(net.activated_layers.len(), 1);
        assert_eq!(net.weights.len(), 1);
        assert_eq!(net.biases.len(), 1);

        assert_eq!(net.layers[0].len(), 3);
        assert_eq!(net.activated_layers[0].len(), 3);
        assert_eq!(net.weights[0].len(), 3);
        assert_eq!(net.biases[0].len(), 3);

        assert_eq!(net.weights[0][0].len(), 2);
        assert_eq!(net.weights[0][1].len(), 2);
        assert_eq!(net.weights[0][2].len(), 2);

        net.inputs = vec![3.0, 2.0];

        net.normalize_input();
        assert_eq!(net.normalized_inputs, vec![3.0, 2.0]);

        net.weights[0] = vec![vec![3.0, 4.0], vec![2.0, 4.0], vec![3.0, 5.0]];
        net.biases[0] = vec![1.0, 1.0, 3.0];

        net.feedforward_layer(0);

        assert_eq!(net.layers[0], vec![18.0, 15.0, 22.0]);
    }

    #[test]
    fn test_feedforward_layer2() {
        let mut net = MultiLayerPerceptron::new(
            vec![2, 3, 3],
            vec![&SIGMOID, &SIGMOID],
            &MSE,
            &NO_NORMALIZATION,
        );
        // test initialization
        assert_eq!(net.layers.len(), 2);
        assert_eq!(net.activated_layers.len(), 2);
        assert_eq!(net.weights.len(), 2);
        assert_eq!(net.biases.len(), 2);

        assert_eq!(net.inputs.len(), 2);
        assert_eq!(net.normalized_inputs.len(), 2);
        // weights connected to inputs
        assert_eq!(net.weights[0][0].len(), 2);
        assert_eq!(net.weights[0][1].len(), 2);
        assert_eq!(net.weights[0][2].len(), 2);

        assert_eq!(net.layers[0].len(), 3);
        assert_eq!(net.activated_layers[0].len(), 3);
        assert_eq!(net.weights[0].len(), 3);
        assert_eq!(net.biases[0].len(), 3);
        // weights connected to 0th layer
        assert_eq!(net.weights[1][0].len(), 3);
        assert_eq!(net.weights[1][1].len(), 3);
        assert_eq!(net.weights[1][2].len(), 3);

        assert_eq!(net.layers[1].len(), 3);
        assert_eq!(net.activated_layers[1].len(), 3);
        assert_eq!(net.weights[1].len(), 3);
        assert_eq!(net.biases[1].len(), 3);

        net.activated_layers[0] = vec![3.0, 3.0, 2.0];
        net.weights[1] = vec![
            vec![3.0, 4.0, 3.0],
            vec![2.0, 4.0, 2.0],
            vec![3.0, 5.0, 2.0],
        ];
        net.biases[1] = vec![1.0, 1.0, 3.0];

        net.feedforward_layer(1);

        assert_eq!(net.layers[1], vec![28.0, 23.0, 31.0]);
    }

    #[test]
    fn test_feedforward_layer_softmax() {
        let mut net =
            MultiLayerPerceptron::new(vec![2, 3], vec![&SOFTMAX], &MSE, &NO_NORMALIZATION);
        // test initialization
        assert_eq!(net.inputs.len(), 2);
        assert_eq!(net.normalized_inputs.len(), 2);

        assert_eq!(net.layers.len(), 1);
        assert_eq!(net.activated_layers.len(), 1);
        assert_eq!(net.weights.len(), 1);
        assert_eq!(net.biases.len(), 1);

        assert_eq!(net.layers[0].len(), 3);
        assert_eq!(net.activated_layers[0].len(), 3);
        assert_eq!(net.weights[0].len(), 3);
        assert_eq!(net.biases[0].len(), 3);

        assert_eq!(net.weights[0][0].len(), 2);
        assert_eq!(net.weights[0][1].len(), 2);
        assert_eq!(net.weights[0][2].len(), 2);

        net.inputs = vec![3.0, 2.0];

        net.normalize_input();
        assert_eq!(net.normalized_inputs, vec![3.0, 2.0]);

        net.weights[0] = vec![vec![3.0, 4.0], vec![2.0, 4.0], vec![3.0, 5.0]];
        net.biases[0] = vec![1.0, 1.0, 3.0];

        net.feedforward_layer(0);

        assert_eq!(net.layers[0], vec![18.0, 15.0, 22.0]);

        let activations = &net.activated_layers[0];
        let expected = vec![
            0.01797011806881207,
            0.0008946794968705339,
            0.9811352024343174,
        ];
        assert!(activations
            .iter()
            .zip(expected)
            .all(|(a, y)| (a - y).abs() < 0.001));
    }

    #[test]
    fn test_feedforward() {
        let mut net =
            MultiLayerPerceptron::new(vec![4, 3, 5], vec![&RELU, &RELU], &MSE, &NO_NORMALIZATION);
        // test initialization
        assert_eq!(net.inputs.len(), 4);
        assert_eq!(net.normalized_inputs.len(), 4);
        // weights connected to inputs
        assert_eq!(net.weights[0][0].len(), 4);
        assert_eq!(net.weights[0][1].len(), 4);
        assert_eq!(net.weights[0][2].len(), 4);

        assert_eq!(net.layers.len(), 2);
        assert_eq!(net.activated_layers.len(), 2);
        assert_eq!(net.weights.len(), 2);
        assert_eq!(net.biases.len(), 2);

        assert_eq!(net.layers[0].len(), 3);
        assert_eq!(net.activated_layers[0].len(), 3);
        assert_eq!(net.weights[0].len(), 3);
        assert_eq!(net.biases[0].len(), 3);
        // weights connected to 0th layer
        assert_eq!(net.weights[1][0].len(), 3);
        assert_eq!(net.weights[1][1].len(), 3);
        assert_eq!(net.weights[1][2].len(), 3);
        assert_eq!(net.weights[1][3].len(), 3);
        assert_eq!(net.weights[1][4].len(), 3);

        assert_eq!(net.layers[1].len(), 5);
        assert_eq!(net.activated_layers[1].len(), 5);
        assert_eq!(net.weights[1].len(), 5);
        assert_eq!(net.biases[1].len(), 5);

        net.weights = vec![
            vec![
                vec![3.0, 2.0, 1.0, 4.0],
                vec![5.0, 1.0, 2.0, 3.0],
                vec![4.0, 1.0, 2.0, 1.0],
            ],
            vec![
                vec![1.0, 2.0, 5.0],
                vec![3.0, 2.0, 1.0],
                vec![2.0, 3.0, 5.0],
                vec![1.0, 4.0, 1.0],
                vec![4.0, 1.0, 2.0],
            ],
        ];
        net.biases = vec![vec![3.0, 1.0, 2.0], vec![3.0, 2.0, 1.0, 2.0, 4.0]];

        net.predict(&vec![2.0, 1.0, 3.0, 4.0]);
        net.feedforward();

        assert_eq!(net.normalized_inputs, vec![2.0, 1.0, 3.0, 4.0]);

        assert_eq!(net.layers[0], vec![30.0, 30.0, 21.0]);
        assert_eq!(net.activated_layers[0], vec![30.0, 30.0, 21.0]);

        assert_eq!(net.layers[1], vec![198.0, 173.0, 256.0, 173.0, 196.0]);
        assert_eq!(
            net.activated_layers[1],
            vec![198.0, 173.0, 256.0, 173.0, 196.0]
        );
    }

    // same net as test_feedforward_layer1
    #[test]
    fn test_backprop_layer() {
        let mut net = MultiLayerPerceptron::new(vec![2, 3], vec![&RELU], &MSE, &NO_NORMALIZATION);

        net.normalized_inputs = vec![3.0, 2.0];

        net.weights[0] = vec![vec![3.0, 4.0], vec![2.0, 4.0], vec![3.0, 5.0]];
        net.biases[0] = vec![1.0, 1.0, 3.0];

        net.feedforward_layer(0);

        assert_eq!(net.activated_layers[0], vec![18.0, 15.0, 22.0]);

        let (dws, dbs, das) = net.backprop_layer(0, None, Some(&vec![15.0, 12.0, 20.0]));
        assert_eq!(das, vec![6.0, 6.0, 4.0]);
        assert_eq!(
            dws,
            vec![vec![18.0, 12.0], vec![18.0, 12.0], vec![12.0, 8.0]]
        );
        assert_eq!(dbs, vec![6.0, 6.0, 4.0]);
    }

    // same net as test_feedforward
    #[test]
    fn test_backprop() {
        let mut net =
            MultiLayerPerceptron::new(vec![4, 3, 5], vec![&RELU, &RELU], &MSE, &NO_NORMALIZATION);

        net.inputs = vec![2.0, 1.0, 3.0, 4.0];
        net.weights = vec![
            vec![
                vec![3.0, 2.0, 1.0, 4.0],
                vec![5.0, 1.0, 2.0, 3.0],
                vec![4.0, 1.0, 2.0, 1.0],
            ],
            vec![
                vec![1.0, 2.0, 5.0],
                vec![3.0, 2.0, 1.0],
                vec![2.0, 3.0, 5.0],
                vec![1.0, 4.0, 1.0],
                vec![4.0, 1.0, 2.0],
            ],
        ];
        net.biases = vec![vec![3.0, 1.0, 2.0], vec![3.0, 2.0, 1.0, 2.0, 4.0]];

        net.feedforward();

        assert_eq!(net.normalized_inputs, vec![2.0, 1.0, 3.0, 4.0]);

        assert_eq!(net.activated_layers[0], vec![30.0, 30.0, 21.0]);

        assert_eq!(
            net.activated_layers[1],
            vec![198.0, 173.0, 256.0, 173.0, 196.0]
        );

        let (dws, dbs) = net.backprop(&vec![180.0, 165.0, 250.0, 170.0, 180.0]);

        // manual calculations
        // dC/da[1] = [36, 16, 12, 6, 32]
        // dC/dw[1][0] = [1080, 1080, 756]
        // dC/dw[1][1] = [480, 480, 336]
        // dC/dw[1][2] = [360, 360, 252]
        // dC/dw[1][3] = [180, 180, 126]
        // dC/dw[1][4] = [960, 960, 672]
        // dC/db[1] = [36, 16, 12, 6, 32]
        //
        // dC/da[0] = [242, 196, 326]
        // dC/dw[0][0] = [484, 242, 726, 968]
        // dC/dw[0][1] = [392, 196, 588, 784]
        // dC/dw[0][2] = [652, 326, 978, 1304]
        // dC/db[0] = [242, 196, 326]

        assert_eq!(dws[1][0], vec![1080.0, 1080.0, 756.0]);
        assert_eq!(dws[1][1], vec![480.0, 480.0, 336.0]);
        assert_eq!(dws[1][2], vec![360.0, 360.0, 252.0]);
        assert_eq!(dws[1][3], vec![180.0, 180.0, 126.0]);
        assert_eq!(dws[1][4], vec![960.0, 960.0, 672.0]);
        assert_eq!(dbs[1], vec![36.0, 16.0, 12.0, 6.0, 32.0]);

        assert_eq!(dws[0][0], vec![484.0, 242.0, 726.0, 968.0]);
        assert_eq!(dws[0][1], vec![392.0, 196.0, 588.0, 784.0]);
        assert_eq!(dws[0][2], vec![652.0, 326.0, 978.0, 1304.0]);
        assert_eq!(dbs[0], vec![242.0, 196.0, 326.0]);
    }

    /*#[test]
    fn test_output() {
        let net = MultiLayerPerceptron::new(vec![3, 3], vec![&SIGMOID], &MSE, &NO_NORMALIZATION);

        net.activated_layers[0] = vec![3.0, 2.0, 1.0];
        let out = net.get_output_str();
        assert_eq!(out, "1");
    }*/
}
