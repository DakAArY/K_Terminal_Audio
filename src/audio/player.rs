use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::io::BufReader;

use crate::audio::fft::{FftInterceptor, SharedSamples};

pub struct AudioPlayer {
    _device_sink: MixerDeviceSink,
    pub player: Player,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let _device_sink = DeviceSinkBuilder::open_default_sink().context("No de pudo inicializar el dispositivo de salida de audio")?;
        let player = Player::connect_new(_device_sink.mixer());

        Ok(Self { _device_sink, player })
    }

    pub fn play_file(&self, path: &str, shared_samples: SharedSamples) -> Result<()> {
        let file = File::open(path).context("No se pudo abrir el archivo de audio")?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).context("No se pudo decodificar")?;

        let interceptor = FftInterceptor::new(decoder, shared_samples);

        self.player.append(interceptor);
        self.player.play();
        Ok(())
    }

    pub fn toggle_playback(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }
}
