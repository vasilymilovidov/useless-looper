use hound::{self, WavReader};
use std::path::Path;

pub enum DecodedSamples {
    F32(Vec<f32>),
    I16(Vec<i16>),
}

pub fn decode_wav<P: AsRef<Path>>(path: P) -> Result<DecodedSamples, hound::Error> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Float => {
            // Handle 32-bit floating-point samples
            let samples: Vec<f32> = reader.samples::<f32>().map(|s| s.unwrap()).collect();
            Ok(DecodedSamples::F32(samples))
        }
        hound::SampleFormat::Int => {
            // Handle 16-bit integer samples
            let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
            Ok(DecodedSamples::I16(samples))
        }
    }
}

pub fn compress_samples<T>(samples: &[T], target_length: usize) -> Vec<f32>
where
    T: Into<f32> + Copy,
{
    let total_samples = samples.len();
    let samples_per_bin = total_samples as f64 / target_length as f64;
    let mut compressed_samples = Vec::with_capacity(target_length);

    for bin in 0..target_length {
        let start = (bin as f64 * samples_per_bin) as usize;
        let end = ((bin as f64 + 1.0) * samples_per_bin).ceil() as usize;
        let slice = &samples[start..end];

        let sum: f32 = slice.iter().map(|&sample| sample.into()).sum();
        let count = slice.len();

        if count > 0 {
            compressed_samples.push(sum / count as f32);
        } else {
            compressed_samples.push(0.0);
        }
    }
    compressed_samples
}
