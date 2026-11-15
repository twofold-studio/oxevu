use super::*;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command, sync::watch};

async fn metadata(
    path: &std::path::Path,
    mut cancel: watch::Receiver<bool>,
) -> std::result::Result<Value, String> {
    let mut command = Command::new("ffprobe");
    command
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-show_entries",
            "stream=codec_type,codec_name,height,color_transfer,pix_fmt:stream_side_data=side_data_type:format=bit_rate",
            "-of",
            "json",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    runtime::parent_death(&mut command);
    let mut child = command
        .spawn()
        .map_err(|_| "ffprobe unavailable".to_string())?;
    let stdout = child.stdout.take().ok_or("ffprobe stdout unavailable")?;
    let mut bytes = Vec::new();
    let result = tokio::select! { result=tokio::time::timeout(std::time::Duration::from_secs(15), async {
        stdout.take(1024 * 1024).read_to_end(&mut bytes).await?;
        child.wait().await
    })
    => result, _=cancel.changed()=> { let _=child.kill().await;let _=child.wait().await;return Err("Playback stopped".into()) } };
    match result {
        Ok(Ok(s)) if s.success() => {
            serde_json::from_slice(&bytes).map_err(|_| "Invalid source metadata".into())
        }
        _ => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err("Source probe failed".into())
        }
    }
}
pub(super) fn command(
    a: &App,
    path: &std::path::Path,
    dir: &std::path::Path,
    start: f64,
    method: &str,
    s: &settings::Settings,
    decode: bool,
) -> Command {
    let mut cmd = Command::new(&a.playback.ffmpeg);
    cmd.args([
        "-nostdin",
        "-v",
        "error",
        "-y",
        "-threads",
        "2",
        "-protocol_whitelist",
        "file,pipe",
        "-ss",
    ])
    .arg(start.to_string());
    if decode {
        cmd.args(["-hwaccel", "cuda", "-hwaccel_device"])
            .arg(s.gpu_index.to_string())
            .args(["-hwaccel_output_format", "cuda"]);
    }
    cmd.arg("-i")
        .arg(path)
        .args(["-map", "0:v:0", "-map", "0:a:0?", "-sn", "-dn"]);
    if method == "remux" {
        cmd.args(["-c:v", "copy"]);
    } else {
        let filter = if decode {
            format!(
                "scale_cuda=w=-2:h='trunc(min(ih,{})/2)*2':format=nv12{}",
                s.max_height,
                if method == "nvenc" {
                    ""
                } else {
                    ",hwdownload,format=nv12"
                }
            )
        } else {
            format!(
                "scale=-2:'trunc(min(ih,{})/2)*2',format=yuv420p",
                s.max_height
            )
        };
        cmd.args([
            "-vf",
            &filter,
            "-c:v",
            if method == "nvenc" {
                "h264_nvenc"
            } else {
                "libx264"
            },
        ]);
        if method == "nvenc" {
            cmd.arg("-gpu")
                .arg(s.gpu_index.to_string())
                .args(["-preset", "p4"]);
        } else {
            cmd.args(["-preset", "veryfast", "-crf"])
                .arg(s.cpu_crf.to_string());
        }
        cmd.args([
            "-maxrate",
            &format!("{}k", (s.max_video_bitrate_mbps * 1000.) as u64),
            "-bufsize",
            &format!("{}k", (s.max_video_bitrate_mbps * 2000.) as u64),
            "-force_key_frames",
            "expr:gte(t,n_forced*2)",
        ]);
    }
    cmd.args([
        "-threads",
        "2",
        "-c:a",
        "aac",
        "-ac",
        "2",
        "-b:a",
        "128k",
        "-max_muxing_queue_size",
        "2048",
        "-avoid_negative_ts",
        "make_zero",
        "-f",
        "hls",
        "-hls_time",
        "2",
        // EVENT keeps the growing playlist anchored at the requested seek offset.
        "-hls_playlist_type",
        "event",
        "-hls_list_size",
        "0",
        "-hls_flags",
        "temp_file",
        "-hls_segment_filename",
    ])
    .arg(dir.join("segment%06d.ts"))
    .arg(dir.join("index.m3u8"));
    cmd
}
#[expect(
    clippy::too_many_arguments,
    reason = "worker owns the immutable session plan and lifecycle channels"
)]
pub async fn run(
    a: App,
    id: String,
    path: PathBuf,
    start: f64,
    mut method: String,
    s: settings::Settings,
    cancel: watch::Receiver<bool>,
    done: watch::Sender<bool>,
) {
    let dir = a.playback.scratch.join(&id);
    let result = async {
        tokio::fs::create_dir(&dir)
            .await
            .map_err(|_| "Cannot create playback scratch".to_owned())?;
        if *cancel.borrow() {
            return Err("Playback stopped".into());
        }
        let meta = metadata(&path, cancel.clone()).await?;
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
            command(&a, &path, &dir, start, &method, &s, decode),
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
            if let Some(e) = a.playback.sessions.lock().await.get_mut(&id) {
                e.value["method"] = json!(method);
                e.value["status"] = json!("starting");
                e.value["decoder"] = json!("cpu");
                e.value["reason"] = json!("GPU pipeline failed; using CPU fallback");
            }
            result = runtime::run_process(
                command(&a, &path, &dir, start, &method, &s, false),
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
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
    let _ = done.send(true);
}
