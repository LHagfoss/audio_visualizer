use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use tokio::sync::mpsc;

pub struct AudioStreamer {
    device: Device,
}

pub struct ActiveAudioStream {
    pub stream: Stream,
    pub sample_rate: f32,
}

impl AudioStreamer {
    pub fn new(device: Device) -> Self {
        AudioStreamer { device }
    }

    pub fn start_capture(&self, tx: mpsc::Sender<Vec<f32>>) -> Result<ActiveAudioStream, String> {
        let config = self
            .device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate as f32;
        let channels = stream_config.channels as usize;
        let err_fn = |err| {
            std::eprintln!("Stream error: {}", err);
        };

        let stream = match sample_format {
            SampleFormat::F32 => self.device.build_input_stream(
                stream_config,
                move |data: &[f32], _| {
                    let samples = data
                        .chunks_exact(channels)
                        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                        .collect();
                    let _ = tx.try_send(samples);
                },
                err_fn,
                None,
            ),
            _ => return Err("Only F32 sample format supported currently".to_string()),
        }
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream playback: {}", e))?;

        Ok(ActiveAudioStream {
            stream,
            sample_rate,
        })
    }
}
