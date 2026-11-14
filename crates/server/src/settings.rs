use crate::{App, Result, bad, db};
use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
    pub session_idle_seconds: u32,
    pub resume_min_percent: f64,
    pub resume_max_percent: f64,
    pub resume_min_duration_seconds: u32,
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
            session_idle_seconds: 120,
            resume_min_percent: 5.,
            resume_max_percent: 90.,
            resume_min_duration_seconds: 300,
        }
    }
}
impl Settings {
    fn validate(&self) -> Result<()> {
        if !matches!(self.hardware_acceleration.as_str(), "none" | "nvenc")
            || self.gpu_index > 31
            || !(1..=16).contains(&self.max_concurrent_transcodes)
            || !self.max_video_bitrate_mbps.is_finite()
            || !(0.25..=200.).contains(&self.max_video_bitrate_mbps)
            || !(144..=4320).contains(&self.max_height)
            || self.cpu_crf > 51
            || !self.scratch_limit_gb.is_finite()
            || !(0.1..=1024.).contains(&self.scratch_limit_gb)
            || !(30..=3600).contains(&self.session_idle_seconds)
            || !self.resume_min_percent.is_finite()
            || !self.resume_max_percent.is_finite()
            || !(0. ..100.).contains(&self.resume_min_percent)
            || !(1. ..=100.).contains(&self.resume_max_percent)
            || self.resume_min_percent >= self.resume_max_percent
            || self.resume_min_duration_seconds > 86400
        {
            return Err(bad("settings outside supported ranges"));
        }
        // Decode is deliberately unavailable until a real decode probe is implemented.
        if self.hardware_decoding {
            return Err(bad("hardware decoding is not available"));
        }
        Ok(())
    }
}
pub async fn load(a: &App) -> Result<Settings> {
    let stored: Option<String> = sqlx::query_scalar("SELECT data FROM server_settings WHERE id=1")
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    stored.map_or_else(
        || Ok(Settings::default()),
        |s| serde_json::from_str(&s).map_err(bad),
    )
}
pub async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    crate::users::admin(&a, &h).await?;
    Ok(Json(
        json!({"settings":load(&a).await?,"capabilities":crate::playback::runtime::capabilities(&a,true).await}),
    ))
}
pub async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Json(settings): Json<Settings>,
) -> Result<Json<Value>> {
    crate::users::admin(&a, &h).await?;
    settings.validate()?;
    sqlx::query("INSERT INTO server_settings(id,data) VALUES(1,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data")
        .bind(serde_json::to_string(&settings).map_err(bad)?)
        .execute(&a.pool).await.map_err(db)?;
    Ok(Json(
        json!({"settings":settings,"capabilities":crate::playback::runtime::capabilities(&a,true).await}),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_decode_is_rejected() {
        assert!(
            Settings {
                hardware_decoding: true,
                ..Default::default()
            }
            .validate()
            .is_err()
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
