pub struct Layer<'a> {
    cost_func: &'a dyn Fn(f32) -> f32,
}

impl Layer<'_> {}
