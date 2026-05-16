use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Track {
    pub path: std::path::PathBuf,
    pub display_name: String,
    pub album: String,
    pub duration_secs: u64,
    pub bitrate: u32,
}

impl Track {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut title = String::new();
        let mut artist = String::new();
        let mut album = String::from("Desconocido");
        let mut duration_secs = 0;
        let mut bitrate = 0;

        if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
            let properties = tagged_file.properties();
            duration_secs = properties.duration().as_secs();
            bitrate = properties.audio_bitrate().unwrap_or(0);

            let tag = tagged_file
                .primary_tag()
                .or_else(|| tagged_file.first_tag());
            if let Some(t) = tag {
                title = t.title().as_deref().unwrap_or("").to_string();
                artist = t.artist().as_deref().unwrap_or("").to_string();
                album = t.album().as_deref().unwrap_or("Desconocido").to_string();
            }
        }
        let display_name = if title.is_empty() {
            file_name
        } else if artist.is_empty() {
            title.clone()
        } else {
            format!("{} - {}", artist, title)
        };

        Self {
            path: path.to_path_buf(),
            display_name,
            album,
            duration_secs,
            bitrate,
        }
    }
}
