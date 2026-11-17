use crate::{Result, bad, settings::Settings};
use serde_json::Value;

// Session preferences may lower an administrator's limits, never raise them.
pub(super) fn apply(value: &Value, settings: &mut Settings) -> Result<bool> {
    let mut explicit = false;
    if let Some(height) = value.get("max_height").filter(|v| !v.is_null()) {
        let height = height
            .as_u64()
            .filter(|h| (144..=4320).contains(h))
            .ok_or_else(|| bad("invalid max_height"))?;
        settings.max_height = settings.max_height.min(height as u32);
        explicit = true;
    }
    if let Some(bitrate) = value.get("video_bitrate_mbps").filter(|v| !v.is_null()) {
        let bitrate = bitrate
            .as_f64()
            .filter(|b| b.is_finite() && (0.25..=200.).contains(b))
            .ok_or_else(|| bad("invalid video_bitrate_mbps"))?;
        settings.max_video_bitrate_mbps = settings.max_video_bitrate_mbps.min(bitrate);
        explicit = true;
    }
    Ok(explicit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn preferences_cannot_raise_server_limits() {
        let mut settings = Settings::default();
        assert!(
            apply(
                &json!({"max_height":2160,"video_bitrate_mbps":100}),
                &mut settings
            )
            .unwrap()
        );
        assert_eq!(settings.max_height, 1080);
        assert_eq!(settings.max_video_bitrate_mbps, 8.);
        assert!(
            apply(
                &json!({"max_height":480,"video_bitrate_mbps":1}),
                &mut settings
            )
            .unwrap()
        );
        assert_eq!(settings.max_height, 480);
        assert_eq!(settings.max_video_bitrate_mbps, 1.);
    }
    #[test]
    fn invalid_preferences_are_rejected() {
        for value in [
            json!({"max_height":-1}),
            json!({"max_height":"720"}),
            json!({"video_bitrate_mbps":0}),
        ] {
            assert!(apply(&value, &mut Settings::default()).is_err());
        }
    }
}
