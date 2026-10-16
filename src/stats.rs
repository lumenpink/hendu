use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct BotStats {
    start_time: Instant,
    audios_transcribed: AtomicU64,
    images_described: AtomicU64,
    stickers_described: AtomicU64,
}

impl Default for BotStats {
    fn default() -> Self {
        Self::new()
    }
}

impl BotStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            audios_transcribed: AtomicU64::new(0),
            images_described: AtomicU64::new(0),
            stickers_described: AtomicU64::new(0),
        }
    }

    pub fn inc_audio(&self) {
        self.audios_transcribed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_image(&self) {
        self.images_described.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_sticker(&self) {
        self.stickers_described.fetch_add(1, Ordering::Relaxed);
    }

    pub fn audios_transcribed(&self) -> u64 {
        self.audios_transcribed.load(Ordering::Relaxed)
    }

    pub fn images_described(&self) -> u64 {
        self.images_described.load(Ordering::Relaxed)
    }

    pub fn stickers_described(&self) -> u64 {
        self.stickers_described.load(Ordering::Relaxed)
    }

    pub fn total_processed(&self) -> u64 {
        self.audios_transcribed() + self.images_described() + self.stickers_described()
    }

    pub fn uptime_formatted(&self) -> String {
        let secs = self.start_time.elapsed().as_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let rem_secs = secs % 60;

        if days > 0 {
            format!("{days}d {hours}h {mins}m")
        } else if hours > 0 {
            format!("{hours}h {mins}m {rem_secs}s")
        } else if mins > 0 {
            format!("{mins}m {rem_secs}s")
        } else {
            format!("{rem_secs}s")
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Áudios transcritos: {}\nImagens descritas: {}\nFigurinhas descritas: {}",
            self.audios_transcribed(),
            self.images_described(),
            self.stickers_described()
        )
    }
}
