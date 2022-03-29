mod lib;

fn main() {
    let config = lib::Config::from_path("benches/tps/config.json").expect("Failed to configure");
    let tps = config.measure().expect("Failed to measure");
    dbg!(&config);
    dbg!(&tps);
}
