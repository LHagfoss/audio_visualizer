mod audio;
mod dsp;

use anyhow::Context;
use audio::device::AudioDeviceManager;
use audio::stream::AudioStreamer;
use dsp::fft::FftProcessor;
use tokio::sync::mpsc;

use crate::dsp::bins::FrequencyBinner;

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

    let fft_size = 2048;
    let num_bars = 40;
    let sample_rate = 48000.0;

    let mut fft_processor = FftProcessor::new(fft_size);
    let binner = FrequencyBinner::new(num_bars, sample_rate, fft_size);
    let mut sample_accumulator: Vec<f32> = Vec::with_capacity(fft_size * 2);

    while let Some(mut raw_samples) = rx.recv().await {
        sample_accumulator.append(&mut raw_samples);

        while sample_accumulator.len() >= fft_size {
            let magnitudes = fft_processor.process(&sample_accumulator[..fft_size]);
            let log_bars = binner.calculate_bins(&magnitudes);

            let visualization: String = log_bars
                .iter()
                .map(|&val| {
                    let height = (val * 35.0) as usize;

                    if height > 0 {
                        "■".repeat(height.min(12))
                    } else {
                        ".".to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");

            println!("{}", visualization);

            sample_accumulator.drain(0..512);
        }
    }

    Ok(())
}
