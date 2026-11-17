//! Validated playback and resume configuration, without HTTP or storage dependencies.
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ValidationError(&'static str);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub transcoding_enabled: bool,
    pub hardware_acceleration: String,
    pub hardware_encoding: bool,
    pub hardware_decoding: bool,
    pub gpu_index: u32,
    pub allow_cpu_fallback: bool,
    pub max_concurrent_transcodes: u32,
    pub max_video_bitrate_mbps: f64,
    pub max_height: u32,
    pub cpu_crf: u32,
    pub scratch_limit_gb: f64,
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,
    #[serde(default = "default_cache_limit")]
    pub cache_limit_gb: f64,
    pub session_idle_seconds: u32,
    pub resume_min_percent: f64,
    pub resume_max_percent: f64,
    pub resume_min_duration_seconds: u32,
}
fn default_cache_enabled() -> bool {
    true
}
fn default_cache_limit() -> f64 {
    5.
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            transcoding_enabled: true,
            hardware_acceleration: "nvenc".into(),
            hardware_encoding: true,
            hardware_decoding: false,
            gpu_index: 0,
            allow_cpu_fallback: true,
            max_concurrent_transcodes: 2,
            max_video_bitrate_mbps: 8.,
            max_height: 1080,
            cpu_crf: 23,
            scratch_limit_gb: 10.,
            cache_enabled: default_cache_enabled(),
            cache_limit_gb: default_cache_limit(),
            session_idle_seconds: 120,
            resume_min_percent: 5.,
            resume_max_percent: 90.,
            resume_min_duration_seconds: 300,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !matches!(self.hardware_acceleration.as_str(), "none" | "nvenc")
            || self.gpu_index > 31
            || !(1..=16).contains(&self.max_concurrent_transcodes)
            || !self.max_video_bitrate_mbps.is_finite()
            || !(0.25..=200.).contains(&self.max_video_bitrate_mbps)
            || !(144..=4320).contains(&self.max_height)
            || self.cpu_crf > 51
            || !self.scratch_limit_gb.is_finite()
            || !(0.1..=1024.).contains(&self.scratch_limit_gb)
            || !self.cache_limit_gb.is_finite()
            || !(0.1..=100.).contains(&self.cache_limit_gb)
            || !(30..=3600).contains(&self.session_idle_seconds)
            || !self.resume_min_percent.is_finite()
            || !self.resume_max_percent.is_finite()
            || !(0. ..100.).contains(&self.resume_min_percent)
            || !(1. ..=100.).contains(&self.resume_max_percent)
            || self.resume_min_percent >= self.resume_max_percent
            || self.resume_min_duration_seconds > 86400
        {
            return Err(ValidationError("settings outside supported ranges"));
        }
        if self.hardware_decoding && self.hardware_acceleration != "nvenc" {
            return Err(ValidationError(
                "hardware decoding requires NVIDIA acceleration",
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nvidia_decode_is_accepted() {
        assert!(
            Settings {
                hardware_decoding: true,
                ..Default::default()
            }
            .validate()
            .is_ok()
        );
    }
    #[test]
    fn defaults_are_valid() {
        assert!(Settings::default().validate().is_ok());
    }
    #[test]
    fn reversed_resume_thresholds_are_rejected() {
        let settings = Settings {
            resume_min_percent: 95.,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }
    #[test]
    fn unbounded_jobs_are_rejected() {
        let settings = Settings {
            max_concurrent_transcodes: 0,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }
}
