pub mod multi_layer_perceptron;

/// weights connected from one neuron to all neuron in the previous layer
/// Vec<f32>
type NeuronWeights = Vec<f32>;

/// Vec<Vec<f32>>
type LayerWeights = Vec<NeuronWeights>;

/// Vec<Vec<Vec<f32>>>
type NetworkWeights = Vec<LayerWeights>;

/// biases in one layer, layer, activated_layer
/// Vec<f32>
type LayerNeurons = Vec<f32>;

/// Vec<Vec<f32>>
type NetworkNeurons = Vec<LayerNeurons>;

/// each element represents number of neurons in the layer
type Shape = Vec<usize>;

/// or initialize
pub trait Resetable {
    fn reset_params(&mut self);
}

/// testing/predicting
pub trait Predictable {
    fn normalize_input(&mut self);

    fn feedforward_layer(&mut self, layer_i: usize);

    fn feedforward(&mut self);
}

pub trait Backpropable {
    /// returns derivatives in order: dC/dw[l], dC/db[l], dC/da[l]
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
        prev_cost_activation_derivatives: Option<&LayerNeurons>,
        expected: Option<&NeuronWeights>,
    ) -> (LayerWeights, LayerNeurons, LayerNeurons);

    /// returns derivatives in order: dC/dw, dC/db
    fn backprop(&self, expected: &NeuronWeights) -> (NetworkWeights, NetworkNeurons);
}

pub trait GradientDescendable: Backpropable + Predictable {
    fn gradient_descent(&mut self);

    fn batch_gradient_descent(&mut self);
}

pub trait Trainable: Predictable {
    fn train(&mut self);
}
