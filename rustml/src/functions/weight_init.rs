// https://machinelearningmastery.com/weight-initialization-for-deep-learning-neural-networks/
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rand_distr::Normal;

pub struct InitFunc<'a> {
    pub func: &'a dyn Fn(usize) -> f32,

    pub description: &'a str,
    /// latex formula
    pub formula: &'a str,
}

/// returns by the xavier initialization method
///  - n: number of inputs in the previous layer
fn xavier(n: usize) -> f32 {
    let upper = 1.0 / f32::sqrt(n as f32);
    let lower = -upper;

    Uniform::from(lower..=upper).sample(&mut thread_rng())
}

pub const XAVIER: InitFunc = InitFunc {
    func: &xavier,
    description: "",
    formula: "",
};

/// returns by the xavier initialization method
///  - n: number of inputs in the previous layer
fn he(n: usize) -> f32 {
    let normal = Normal::new(0.0, f32::sqrt(2.0 / (n as f32))).unwrap();

    normal.sample(&mut thread_rng())
}

pub const HE: InitFunc = InitFunc {
    func: &he,
    description: "",
    formula: "",
};
