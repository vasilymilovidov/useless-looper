pub fn scale_value(input: f64, input_range: (f64, f64), output_range: (f64, f64)) -> f64 {
    let (input_min, input_max) = input_range;
    let (output_min, output_max) = output_range;

    let clamped_input = input.clamp(input_min, input_max);

    let proportion = (clamped_input - input_min) / (input_max - input_min);

    output_min + proportion * (output_max - output_min)
}

pub fn calculate_spacing(new_width: f32, num_samples: i32, samples_width: f32) -> f32 {
    let total_width = num_samples as f32 * samples_width;
    let num_gaps = (num_samples - 1) as f32;
    if num_gaps > 0.0 {
        (new_width - total_width) / num_gaps
    } else {
        0.0
    }
}

pub fn scale_values_to_unit_range(values: Vec<f32>) -> Vec<f32> {
    let (min_val, max_val) = values.iter().cloned().fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), val| {
        (min.min(val), max.max(val))
    });

    if min_val >= max_val {
        return Vec::new();
    }

    values
        .into_iter()
        .map(|x| (x - min_val) / (max_val - min_val))
        .collect()
}
