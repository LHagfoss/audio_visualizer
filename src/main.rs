mod audio;
mod dsp;
mod ui;

use audio::device::AudioDeviceManager;
use audio::stream::AudioStreamer;
use dsp::bins::FrequencyBinner;
use dsp::fft::FftProcessor;
use dsp::smoother::SpectrumSmoother;
use dsp::state::{AppState, AppView};

use crossterm::ExecutableCommand;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use std::io::stdout;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = AudioDeviceManager::new();
    let devices = manager.list_devices().map_err(|e| anyhow::anyhow!(e))?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let app_state = AppState::new(32);
    let mut list_state = ListState::default();
    if !devices.is_empty() {
        list_state.select(Some(0));
    }

    let mut render_interval = tokio::time::interval(std::time::Duration::from_millis(16));
    let mut active_stream_handle: Option<cpal::Stream> = None;

    loop {
        tokio::select! {
            _ = render_interval.tick() => {
                terminal.draw(|f| {
                    ui::render(f, &app_state, &devices, &mut list_state);
                })?;
            }
        }

        if crossterm::event::poll(std::time::Duration::from_millis(0))?
            && let Event::Key(key_event) = crossterm::event::read()?
        {
            match app_state.get_view() {
                AppView::SelectDevice => match key_event.code {
                    KeyCode::Up => {
                        if let Some(selected) = list_state.selected()
                            && selected > 0
                        {
                            list_state.select(Some(selected - 1));
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = list_state.selected()
                            && selected < devices.len() - 1
                        {
                            list_state.select(Some(selected + 1));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = list_state.selected() {
                            let device_idx = devices[selected].0;
                            let device = manager
                                .get_device_by_index(device_idx)
                                .map_err(|e| anyhow::anyhow!(e))?;

                            let (tx, mut rx) = mpsc::channel::<Vec<f32>>(128);
                            let streamer = AudioStreamer::new(device);
                            let active_stream =
                                streamer.start_capture(tx).map_err(|e| anyhow::anyhow!(e))?;
                            let sample_rate = active_stream.sample_rate;
                            active_stream_handle = Some(active_stream.stream);

                            let processing_state = app_state.clone();
                            tokio::spawn(async move {
                                let fft_size = 2048;
                                let mut fft_processor = FftProcessor::new(fft_size);
                                let mut binner = FrequencyBinner::new(
                                    processing_state.get_num_bars(),
                                    sample_rate,
                                    fft_size,
                                );
                                let mut smoother =
                                    SpectrumSmoother::new(processing_state.get_response());
                                let mut sample_accumulator = Vec::with_capacity(fft_size * 2);

                                while let Some(mut raw_samples) = rx.recv().await {
                                    sample_accumulator.append(&mut raw_samples);

                                    while sample_accumulator.len() >= fft_size {
                                        let magnitudes =
                                            fft_processor.process(&sample_accumulator[..fft_size]);
                                        let current_bars = processing_state.get_num_bars();
                                        if binner.num_bars() != current_bars {
                                            binner = FrequencyBinner::new(
                                                current_bars,
                                                sample_rate,
                                                fft_size,
                                            );
                                        }
                                        let log_bars = binner.calculate_bins(&magnitudes);
                                        smoother.set_response(processing_state.get_response());
                                        let smoothed_bars = smoother.process(log_bars);

                                        processing_state.update_bins(smoothed_bars);
                                        sample_accumulator.drain(0..512);
                                    }
                                }
                            });

                            app_state.set_view(AppView::Visualizer);
                        }
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                },
                AppView::Visualizer => match key_event.code {
                    KeyCode::Up => {
                        app_state.adjust_bars(2);
                    }
                    KeyCode::Down => {
                        app_state.adjust_bars(-2);
                    }
                    KeyCode::Right => app_state.adjust_response(0.10),
                    KeyCode::Left => app_state.adjust_response(-0.10),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                },
            }
        }
    }

    drop(active_stream_handle);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
