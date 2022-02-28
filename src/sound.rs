use std::{fs::File, iter::Map, path::Path};

use hound::WavIntoSamples;
use itertools::{Itertools, Tuples};

type ConvertFn = fn(Result<i32, hound::Error>) -> i32;
type SampleIterator = Tuples<Map<hound::WavIntoSamples<File, i32>, ConvertFn>, (i32, i32)>;

pub struct WavStreamer {
    pub spec: hound::WavSpec,
    samples: SampleIterator,
}

impl WavStreamer {
    pub fn new(filename: &str) -> Self {
        let inp_file = File::open(Path::new(filename)).unwrap();
        let wav_reader = hound::WavReader::new(inp_file).unwrap();
        let spec = wav_reader.spec();
        let samples = wav_reader
            .into_samples::<i32>()
            .map((|i| i.unwrap()) as ConvertFn)
            .tuples();
        Self { spec, samples }
    }

    pub fn iter(&mut self) -> &mut SampleIterator {
        &mut self.samples
    }
}
