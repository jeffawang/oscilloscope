use std::{fs::File, path::Path};

pub struct WavStreamer {
    spec: hound::WavSpec,
    samples: hound::WavIntoSamples<File, i16>,
}

impl WavStreamer {
    pub fn new(filename: &str) -> Self {
        let inp_file = File::open(Path::new(filename)).unwrap();
        let wav_reader = hound::WavReader::new(inp_file).unwrap();
        let spec = wav_reader.spec();
        let samples = wav_reader.into_samples::<i16>();
        return Self {
            spec: spec,
            samples: samples,
        };
    }
}

impl Iterator for WavStreamer {
    type Item = [f32; 2];
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.samples.next()?.unwrap() as f32 / i16::MAX as f32;
        let y = self.samples.next()?.unwrap() as f32 / i16::MAX as f32;
        Some([x, y])
    }
}
