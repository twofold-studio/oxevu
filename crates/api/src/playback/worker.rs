use super::*;
use tokio::{process::Command, sync::watch};
pub(super) fn command(
    a: &App,
    input: &source::Input,
    dir: &std::path::Path,
    start: f64,
    method: &str,
    s: &settings::Settings,
    decode: bool,
) -> Command {
    let mut command = Command::new(&a.playback.ffmpeg);
    command.args(["-nostdin", "-v", "error", "-y", "-threads", "2"]);
    input.arguments(&mut command);
    oxevu_playback::command::configure(command, input.path(), dir, start, method, s, decode)
}
#[expect(
    clippy::too_many_arguments,
    reason = "worker owns the immutable session plan and lifecycle channels"
)]
pub async fn run(
    a: App,
    id: String,
    source: source::Source,
    start: f64,
    mut method: String,
    s: settings::Settings,
    cancel: watch::Receiver<bool>,
    done: watch::Sender<bool>,
) {
    let dir = a.playback.scratch.join(&id);
    let requested_method = method.clone();
    let cache_key = match source.local().filter(|_| s.cache_enabled) {
        Some(path) => super::cache::key(&a, path, &requested_method, &s).await,
        None => None,
    };
    let mut output = json!({});
    let result = async {
        tokio::fs::create_dir(&dir)
            .await
            .map_err(|_| "Cannot create playback scratch".to_owned())?;
        if *cancel.borrow() {
            return Err("Playback stopped".into());
        }
        let input=source.prepare(&a).await?;
        let meta = if input.progressive() {
            super::source_probe::progressive(&input,cancel.clone()).await?
        } else {
            super::source_probe::metadata(&a,input.path(),cancel.clone()).await?
        };
        if input.progressive() {
            let duration=meta["format"]["duration"].as_str().and_then(|s|s.parse::<f64>().ok()).filter(|v|v.is_finite()&&*v>0.);
            if duration.is_some_and(|d|start>=d){return Err("Start offset exceeds duration".into());}
            let item = {
                let mut sessions=a.playback.sessions.lock().await;
                sessions.get_mut(&id).map(|e|{e.value["duration_seconds"]=json!(duration);e.item.clone()})
            };
            if let (Some(item),Some(duration))=(item,duration) {
                sqlx::query("UPDATE items SET data=json_set(data,'$.duration_seconds',?) WHERE id=? AND EXISTS(SELECT 1 FROM acquisition_stream_items WHERE item_id=items.id)")
                    .bind(duration).bind(item).execute(&a.pool).await.map_err(|_|"Cannot retain source duration")?;
            }
        }
        let streams = meta["streams"].as_array().ok_or("Source has no streams")?;
        if streams.iter().any(|v| {
            matches!(
                v["color_transfer"].as_str(),
                Some("smpte2084" | "arib-std-b67")
            ) || v["codec_name"] == "dolby_vision"
                || v["side_data_list"].as_array().is_some_and(|list| {
                    list.iter().any(|d| {
                        d["side_data_type"]
                            .as_str()
                            .is_some_and(|s| s.contains("DOVI"))
                    })
                })
        }) {
            return Err(
                "HDR conversion is unsupported; use a compatible direct-play client".into(),
            );
        }
        if method == "remux" {
            let compatible = streams.iter().any(|v| {
                v["codec_type"] == "video"
                    && v["codec_name"] == "h264"
                    && v["height"]
                        .as_u64()
                        .is_some_and(|h| h <= u64::from(s.max_height))
                    && v["pix_fmt"] == "yuv420p"
            }) && meta["format"]["bit_rate"]
                .as_str()
                .and_then(|v| v.parse::<f64>().ok())
                .is_some_and(|b| b <= s.max_video_bitrate_mbps * 1_000_000.)
                && start == 0.;
            if !compatible {
                method = if s.hardware_acceleration == "nvenc" && s.hardware_encoding {
                    "nvenc"
                } else {
                    "cpu"
                }
                .into();
            }
        }
        let supported_decode = streams.iter().any(|v| {
            v["codec_type"] == "video"
                && matches!(
                    v["codec_name"].as_str(),
                    Some("h264" | "hevc" | "mpeg2video" | "vc1" | "vp8" | "vp9" | "av1")
                )
        });
        let decode = s.hardware_decoding && method != "remux" && supported_decode;
        let video = streams.iter().find(|v|v["codec_type"]=="video");
        let dimensions = video.and_then(|v|Some((v["width"].as_u64()?,v["height"].as_u64()?)));
        let dimensions = dimensions.filter(|(_,h)|*h>0).map(|(w,h)| {
            if method=="remux" {(w,h)} else {
                let height=h.min(u64::from(s.max_height))/2*2;
                let width=((w as f64*height as f64/h as f64)/2.).round() as u64*2;
                (width,height)
            }
        });
        output=json!({"method":method,"decoder":if method=="remux"{"copy"}else if decode{"cuda"}else{"cpu"},"width":dimensions.map(|d|d.0),"height":dimensions.map(|d|d.1)});
        if s.hardware_decoding && method != "remux" && !supported_decode && !s.allow_cpu_fallback {
            return Err("Source codec is unsupported by CUDA decoding".into());
        }
        let limit = (s.scratch_limit_gb * 1024. * 1024. * 1024.) as u64;
        if runtime::disk_usage(&a.playback.scratch)
            .await
            .unwrap_or(u64::MAX)
            >= limit
        {
            return Err("Scratch storage limit reached".into());
        }
        if let Some(e) = a.playback.sessions.lock().await.get_mut(&id) {
            e.value["width"]=output["width"].clone(); e.value["height"]=output["height"].clone();
            e.value["method"] = json!(method);
            e.value["decoder"] = json!(if method == "remux" {
                "copy"
            } else if decode {
                "cuda"
            } else {
                "cpu"
            });
            if s.hardware_decoding && !supported_decode && method != "remux" {
                e.value["reason"] = json!("Source codec unsupported by CUDA; using CPU decoding");
            }
        }
        let mut result = runtime::run_process(
            command(&a, &input, &dir, start, &method, &s, decode),
            cancel.clone(),
            &a.playback.scratch,
            limit,
            &dir.join("index.m3u8"),
            &a,
            &id,
        )
        .await;
        // One GPU retry at the same absolute offset, never an unbounded retry loop.
        if (method == "nvenc" || decode)
            && s.allow_cpu_fallback
            && result.as_ref().is_err_and(|e| {
                e == "FFmpeg encoding failed"
                    || e == "FFmpeg could not start"
                    || e == "FFmpeg startup timed out"
            })
            && !*cancel.borrow()
        {
            let _ = tokio::fs::remove_dir_all(&dir).await;
            tokio::fs::create_dir(&dir)
                .await
                .map_err(|_| "Cannot reset playback scratch")?;
            method = "cpu".into();
            output["method"]=json!("cpu"); output["decoder"]=json!("cpu");
            if let Some(e) = a.playback.sessions.lock().await.get_mut(&id) {
                e.value["method"] = json!(method);
                e.value["status"] = json!("starting");
                e.value["decoder"] = json!("cpu");
                e.value["reason"] = json!("GPU pipeline failed; using CPU fallback");
            }
            result = runtime::run_process(
                command(&a, &input, &dir, start, &method, &s, false),
                cancel.clone(),
                &a.playback.scratch,
                limit,
                &dir.join("index.m3u8"),
                &a,
                &id,
            )
            .await;
        }
        if result.is_ok()
            && !tokio::fs::try_exists(dir.join("index.m3u8"))
                .await
                .unwrap_or(false)
        {
            return Err("FFmpeg produced no playable output".into());
        }
        if result.is_ok()
            && runtime::disk_usage(&a.playback.scratch)
                .await
                .unwrap_or(u64::MAX)
                >= limit
        {
            return Err("Scratch storage limit reached".into());
        }
        result
    }
    .await;
    if let Some(e) = a.playback.sessions.lock().await.get_mut(&id) {
        match &result {
            Ok(()) => e.value["status"] = json!("ready"),
            Err(error) => {
                e.value["status"] = json!("failed");
                e.value["error"] = json!(error);
                e.value["stream_url"] = Value::Null;
            }
        }
    }
    if let Some(path) = source.local()
        && (result.is_ok()
            || result
                .as_ref()
                .is_err_and(|error| error == "Playback stopped"))
    {
        super::cache::store(
            &a,
            &id,
            path,
            start,
            &requested_method,
            &s,
            &output,
            cache_key.as_deref(),
        )
        .await;
    }
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
    let _ = done.send(true);
}
