//! One-shot benchmark by counting the CPU instructions and memory accesses
//! using [iai](https://github.com/bheisler/iai)
//! for checking performance improvements or regressions in CI environments

use iai::black_box;

use crate::lib::Config;

mod lib;

impl Config {
    fn bench(self) {
        // WIP limit the measurement range
        let _ = self.measure().expect("Failed to measure");
    }
}

fn iai_with_config() {
    let config = Config::from_path("benches/tps/config.json").expect("Failed to configure");
    dbg!(&config);
    black_box(config).bench();
}

iai::main!(iai_with_config);
