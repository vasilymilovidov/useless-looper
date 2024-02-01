use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;

pub fn _record() {
    let host = cpal::default_host();
        let device = host.default_input_device().expect("Failed to get default input device");
        let config = device.default_input_config().expect("Failed to get default input config");
        let sample_format = config.sample_format();

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create("recorded.wav", spec).unwrap();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(&config.into(), move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let amplitude = (sample * std::i16::MAX as f32) as i16;
                    writer.write_sample(amplitude).unwrap();
                }
            }, err_fn, None),
            _ => unimplemented!(),
        }.unwrap();

        stream.play().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));

        stream.pause().unwrap();
}
