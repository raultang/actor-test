use metrics_runtime::Receiver;
use metrics_runtime::observers::YamlBuilder;
use log::Level;
use std::time::Duration;
use std::thread;
use metrics_exporter_log::LogExporter;

pub fn init_log() {
    log4rs::init_file("./resources/log4rs.yml", Default::default()).unwrap();
}

pub fn into_metrics() {
    let receiver = Receiver::builder()
        .build()
        .expect("failed to create receiver");

    let mut exporter = LogExporter::new(
        receiver.get_controller(),
        YamlBuilder::new(),
        Level::Info,
        Duration::from_secs(10),
    );

    receiver.install();

    thread::spawn(move || {
        exporter.run();
    });
}