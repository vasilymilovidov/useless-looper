use hound;
use std::path::Path;

pub fn decode_wav<P: AsRef<Path>>(path: P) -> Result<Vec<f32>, hound::Error> {
    let mut reader = hound::WavReader::open(path)?;
    let samples =reader.samples::<f32>().map(|s| s.unwrap()).collect();

    Ok(samples)
}

pub fn compress_samples(samples: &[f32], target_length: usize) -> Vec<f32> {
    let total_samples = samples.len();
    let samples_per_bin = total_samples as f64 / target_length as f64;
    let mut compressed_samples = Vec::with_capacity(target_length);

    for bin in 0..target_length {
        let start = (bin as f64 * samples_per_bin) as usize;
        let end = ((bin as f64 + 1.0) * samples_per_bin).ceil() as usize;

        let sum: f32 = samples[start..end].iter().sum();
        let count = end - start;
        compressed_samples.push(sum / count as f32);
    }

    compressed_samples
}
