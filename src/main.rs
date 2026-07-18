mod audio;
mod dsp;

use anyhow::Context;
use audio::device::AudioDeviceManager;
use audio::stream::AudioStreamer;
use dsp::fft::FftProcessor;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = AudioDeviceManager::new();

    println!("=== Checking Audio Hardware ===");
    let devices = manager.list_devices().map_err(|e| anyhow::anyhow!(e))?;

    for (idx, name) in &devices {
        println!("[{}] {}", idx, name);
    }

    let mut input = String::new();
    println!("\nEnter BlackHole device index:");
    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input line")?;
    let target_idx: usize = input.trim().parse().unwrap_or(0);

    let device = manager
        .get_device_by_index(target_idx)
        .map_err(|e| anyhow::anyhow!(e))?;
    let (tx, mut rx) = mpsc::channel::<Vec<f32>>(128);

    let streamer = AudioStreamer::new(device);
    let _stream = streamer.start_capture(tx).map_err(|e| anyhow::anyhow!(e))?;

    println!("\nStreaming live PCM data... Processing FFT bins down channel...");

    let mut fft_processor = FftProcessor::new(2048);

    while let Some(raw_samples) = rx.recv().await {
        let max_amplitude = raw_samples
            .iter()
            .fold(0.0f32, |max, &val| max.max(val.abs()));

        println!(
            "Buffer length: {}, Peak amplitude: {:.4}",
            raw_samples.len(),
            max_amplitude
        );
    }

    Ok(())
}
