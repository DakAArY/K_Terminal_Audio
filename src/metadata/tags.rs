use std::path::Path;
use id3::{Tag, TagLike};
use metaflac::Tag as FlacTag;

#[derive(Clone, Debug)]
pub struct Track {
    pub path: std::path::PathBuf,

    pub display_name: String,
}

impl Track {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();

        let mut title = String::new();
        let mut artist = String::new();

        if ext == "mp3" {
            if let Ok(tag) = Tag::read_from_path(path) {
                title = tag.title().unwrap_or("").to_string();
                artist = tag.artist().unwrap_or("").to_string();           
            }
        } else if ext == "flac" {
            if let Ok(tag) = FlacTag::read_from_path(path) {
                if let Some(vorbis) = tag.vorbis_comments() {
                    title = vorbis.title().and_then(|t| t.first().cloned()).unwrap_or_default();
                    artist = vorbis.artist().and_then(|a| a.first().cloned()).unwrap_or_default();
                    
                }
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
        }
    }
}
