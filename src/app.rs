use crate::{audio::player::AudioPlayer};
use crate::metadata::tags::Track;
use crate::audio::fft::SharedSamples;
use rustfft::{FftPlanner, num_complex::Complex};
use core::f32;
use std::sync::{Arc, Mutex};
use ratatui::{widgets::ListState};
use walkdir::WalkDir;
use lofty::probe::Probe;
use lofty::prelude::*;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

#[derive(PartialEq)]
pub enum VisMode {
    Spectrum,
    Oscilloscope,
}

pub struct App {
    pub quit: bool,
    pub player: AudioPlayer,
    pub items: Vec<Track>,
    pub state: ListState,
    pub current_track_name: String,
    pub loop_track: bool,
    pub shared_samples: SharedSamples,
    pub fft_bars: Vec<u64>,
    pub vis_mode: VisMode,
    pub oscilloscope_data: Vec<(f64, f64)>,
    pub cover_art: Option<StatefulProtocol>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let mut state = ListState::default();
        state.select(Some(0));

        let buffer_size = 4096;
        let shared_samples = Arc::new(Mutex::new((vec![0.0; buffer_size], 0)));

        Ok(Self { 
            quit: false,
            player: AudioPlayer::new()?,
            items: Vec::new(),
            state,
            current_track_name: String::from("Ninguna pista cargada"),
            loop_track: false,
            shared_samples,
            fft_bars: vec![0; 24],
            vis_mode: VisMode::Spectrum,
            oscilloscope_data: Vec::new(),
            cover_art: None,
        })
    }

    pub fn load_cover_art(&mut self, file_path: &str) {
        self.cover_art = None;

        if let Ok(tagged_file) = Probe::open(file_path).and_then(|p| p.read()) {
            let tags = tagged_file.primary_tag().into_iter().chain(tagged_file.tags());

            for tag in tags {
                if let Some(pic) = tag.pictures().first() {
                    if let Ok(img) = image::load_from_memory(pic.data()) {
                        let rgb_img = img.to_rgb8();
                        let dynamic_img = image::DynamicImage::ImageRgb8(rgb_img);
                        if let Ok(picker) = Picker::from_query_stdio() {
                            self.cover_art = Some(picker.new_resize_protocol(dynamic_img));
                        }
                        
                        break;
                    }
                }
            }
        }
    }

    pub fn toggle_visualizer(&mut self) {
        self.vis_mode = match self.vis_mode {
            VisMode::Spectrum => VisMode::Oscilloscope,
            VisMode::Oscilloscope => VisMode::Spectrum,
        };
    }

    pub fn scan_directory(&mut self, path: &str) {
        self.items = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_owned())
        .filter(|path| {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                ext == "mp3" || ext == "flac"
            })
        .map(|path| Track::from_path(&path))
        .collect();
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 { self.items.len().saturating_sub(1) } else { i - 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn play_selected(&mut self) -> anyhow::Result<()> {
        let track_data = if let Some(i) = self.state.selected() {
            self.items.get(i).map(|track| {
                (
                    track.path.to_str().unwrap_or("").to_string(),
                    track.display_name.clone()
                )
            })
        } else {
            None
        };

        if let Some((path_str, display_name)) = track_data {
            self.load_cover_art(&path_str);

            self.player.player.stop();
            self.player.play_file(&path_str, Arc::clone(&self.shared_samples))?;
            self.current_track_name = display_name;
        }

        Ok(())
    }

    pub fn update_visualizer(&mut self) {
        let buffer_size = 2048;
        let mut samples = vec![0.0; buffer_size];

        if let Ok(guard) = self.shared_samples.lock() {
            let cursor = guard.1;
            for i in 0..buffer_size {
                samples[i] = guard.0[(cursor + i) % buffer_size];
            }
        }

        match self.vis_mode {
            VisMode::Spectrum => {
                for i in 0..buffer_size {
                    let multiplier = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (buffer_size - 1) as f32).cos());
                    samples[i] *= multiplier;
                }

                let mut planner = FftPlanner::new();
                let fft = planner.plan_fft_forward(buffer_size);
                let mut buffer: Vec<Complex<f32>> = samples.into_iter().map(|f| Complex { re: f, im: 0.0 }).collect();
                fft.process(&mut buffer);

                let num_bins = 24;
                let mut bins = vec![0.0; num_bins];
                let usable_bins = buffer_size / 2;
                let bin_size = usable_bins / num_bins;

                for i in 0..num_bins {
                    let start = i * bin_size;
                    let end = start + bin_size;
                    let mut sum = 0.0;
                    for j in start..end {
                        let mag = (buffer[j].re.powi(2) + buffer[j].im.powi(2)).sqrt();
                        sum += mag;
                    }
                    bins[i] = sum / bin_size as f32
                }

                self.fft_bars.clear();
                for i in 0..num_bins {
                    let mut height = (bins[i] * 50.0) as u64;
                    if height > 100 { height = 100; }
                    self.fft_bars.push(height);
                }
            }
            VisMode::Oscilloscope => {
                self.oscilloscope_data.clear();
                for i in 0..1024 {
                    self.oscilloscope_data.push((i as f64, samples[i] as f64));
                }
            }
        }
    }

    pub fn play_next(&mut self) -> anyhow::Result<()> {
        self.next();
        self.play_selected()
    }

    pub fn play_previous(&mut self) -> anyhow::Result<()> {
        self.previous();
        self.play_selected()
    }

    pub fn toggle_loop(&mut self) {
        self.loop_track = !self.loop_track;
    }
}
