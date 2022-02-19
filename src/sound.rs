use std::{fs::File, iter::Map, path::Path};

use hound::{WavIntoSamples, WavSamples};
use itertools::{Itertools, Tuples};

type ConvertFn = fn(Result<i16, hound::Error>) -> f32;
type SampleIterator = Tuples<Map<WavIntoSamples<File, i16>, ConvertFn>, (f32, f32)>;

pub struct WavStreamer {
    spec: hound::WavSpec,
    samples: SampleIterator,
}

impl WavStreamer {
    pub fn new(filename: &str) -> Self {
        let inp_file = File::open(Path::new(filename)).unwrap();
        let wav_reader = hound::WavReader::new(inp_file).unwrap();
        let spec = wav_reader.spec();
        let samples = wav_reader
            .into_samples::<i16>()
            .map((|i| (i.unwrap() as f32) / (i16::MAX as f32)) as ConvertFn)
            .tuples();
        Self { spec, samples }
    }

    pub fn iter(&mut self) -> &mut SampleIterator {
        &mut self.samples
    }
}
