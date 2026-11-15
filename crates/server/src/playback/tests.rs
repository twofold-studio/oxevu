use super::*;
async fn fixture() -> (App, HeaderMap, PathBuf) {
    let a = crate::tests::app().await;
    let h = crate::tests::account(&a, "owner", true).await;
    let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir(&root).await.unwrap();
    let file = root.join("source.mkv");
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=24",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440",
            "-t",
            "8",
            "-c:v",
            "libx264",
            "-threads",
            "1",
            "-c:a",
            "aac",
        ])
        .arg(&file)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    sqlx::query("INSERT INTO libraries VALUES('l','Test',?,'movies')")
        .bind(root.to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l',?,?)")
        .bind(file.to_string_lossy().as_ref())
        .bind(
            json!({"id":"i","library_id":"l","duration_seconds":8,"video_codec":"h264"})
                .to_string(),
        )
        .execute(&a.pool)
        .await
        .unwrap();
    (a, h, root)
}
async fn plan(a: &App, hardware: &str, gpu: u32, fallback: bool) {
    let s = settings::Settings {
        hardware_acceleration: hardware.into(),
        gpu_index: gpu,
        allow_cpu_fallback: fallback,
        ..Default::default()
    };
    sqlx::query(
        "INSERT INTO server_settings VALUES(1,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
    )
    .bind(serde_json::to_string(&s).unwrap())
    .execute(&a.pool)
    .await
    .unwrap();
}
async fn start(a: &App, h: &HeaderMap, offset: f64, mode: &str) -> Value {
    create(
        State(a.clone()),
        h.clone(),
        Json(
            json!({"item_id":"i","start_seconds":offset,"mode":mode,"direct_play_supported":false}),
        ),
    )
    .await
    .unwrap()
    .0["session"]
        .clone()
}
async fn finished(a: &App, h: &HeaderMap, id: &str) -> Value {
    tokio::time::timeout(std::time::Duration::from_secs(45), async {
        loop {
            let value = get(State(a.clone()), h.clone(), Path(id.into()))
                .await
                .unwrap()
                .0["session"]
                .clone();
            let done = a
                .playback
                .sessions
                .lock()
                .await
                .get(id)
                .unwrap()
                .done
                .clone();
            if *done.borrow() {
                return value;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}
async fn readable(a: &App, h: &HeaderMap, id: &str) {
    let response = hls(
        State(a.clone()),
        h.clone(),
        Path((id.into(), "index.m3u8".into())),
        Request::new(axum::body::Body::empty()),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/vnd.apple.mpegurl"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let playlist = std::str::from_utf8(&body).unwrap();
    assert!(playlist.contains("#EXT-X-PLAYLIST-TYPE:EVENT"));
    assert!(playlist.contains("#EXT-X-START:TIME-OFFSET=0,PRECISE=YES"));

    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_name:format=duration",
            "-of",
            "json",
        ])
        .arg(a.playback.scratch.join(id).join("index.m3u8"))
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    let decode = tokio::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(a.playback.scratch.join(id).join("index.m3u8"))
        .args(["-f", "null", "-"])
        .status()
        .await
        .unwrap();
    assert!(decode.success());
    let v = String::from_utf8_lossy(&output.stdout);
    assert!(v.contains("h264"));
    assert!(v.contains("aac"));
    let mut entries = tokio::fs::read_dir(a.playback.scratch.join(id))
        .await
        .unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let file = entry.file_name().to_string_lossy().into_owned();
        if file.ends_with(".ts") {
            let segment = hls(
                State(a.clone()),
                h.clone(),
                Path((id.into(), file)),
                Request::new(axum::body::Body::empty()),
            )
            .await
            .unwrap();
            assert_eq!(segment.status(), StatusCode::OK);
            return;
        }
    }
    panic!("no playable segment");
}
#[tokio::test]
async fn cpu_offset_ownership_live_permissions_and_stop() {
    let (a, h, root) = fixture().await;
    plan(&a, "none", 0, true).await;
    let s = start(&a, &h, 3., "transcode").await;
    let id = s["id"].as_str().unwrap();
    let s = finished(&a, &h, id).await;
    assert_eq!(s["status"], "ready");
    assert_eq!(s["method"], "cpu");
    assert_eq!(s["start_seconds"], 3.);
    readable(&a, &h, id).await;
    let duration = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "json",
        ])
        .arg(a.playback.scratch.join(id).join("index.m3u8"))
        .output()
        .await
        .unwrap();
    let d: Value = serde_json::from_slice(&duration.stdout).unwrap();
    let duration = d["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((4.5..5.5).contains(&duration), "offset duration {duration}");
    let guest = crate::tests::account(&a, "guest", true).await;
    assert!(
        hls(
            State(a.clone()),
            guest,
            Path((id.into(), "index.m3u8".into())),
            Request::new(axum::body::Body::empty())
        )
        .await
        .is_err()
    );
    sqlx::query("UPDATE user_access SET can_transcode=0 WHERE user_id='owner'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        heartbeat(State(a.clone()), h.clone(), Path(id.into()))
            .await
            .is_err()
    );
    sqlx::query("UPDATE user_access SET can_transcode=1 WHERE user_id='owner'")
        .execute(&a.pool)
        .await
        .unwrap();
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    assert!(!a.playback.scratch.join(id).exists());
    let s = start(&a, &h, 5., "transcode").await;
    let id = s["id"].as_str().unwrap();
    assert_eq!(finished(&a, &h, id).await["start_seconds"], 5.);
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}
#[tokio::test]
#[ignore = "requires NVIDIA GPU0; run explicitly on vd"]
async fn nvenc_real_encode_and_gpu_failure_fallback() {
    let (a, h, root) = fixture().await;
    plan(&a, "nvenc", 0, false).await;
    let cap = runtime::capabilities(&a, true).await;
    assert_eq!(
        cap["nvenc_available"], true,
        "vd GPU0 must support actual encode"
    );
    let s = start(&a, &h, 0., "transcode").await;
    let id = s["id"].as_str().unwrap();
    let out = finished(&a, &h, id).await;
    assert_eq!(out["status"], "ready", "{out}");
    assert_eq!(out["method"], "nvenc");
    readable(&a, &h, id).await;
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    plan(&a, "nvenc", 31, true).await;
    let s = start(&a, &h, 0., "transcode").await;
    let out = finished(&a, &h, s["id"].as_str().unwrap()).await;
    assert_eq!(out["status"], "ready", "{out}");
    assert_eq!(out["method"], "cpu");
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}
#[tokio::test]
async fn direct_remux_and_failed_gpu_cleanup() {
    let (a, h, root) = fixture().await;
    plan(&a, "none", 0, false).await;
    let direct = start(&a, &h, 2., "direct").await;
    assert_eq!(direct["method"], "direct");
    let remux = start(&a, &h, 0., "auto").await;
    let id = remux["id"].as_str().unwrap();
    let out = finished(&a, &h, id).await;
    assert_eq!(out["method"], "remux");
    assert_eq!(out["status"], "ready");
    readable(&a, &h, id).await;
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    plan(&a, "nvenc", 31, false).await;
    let s = start(&a, &h, 0., "transcode").await;
    let id = s["id"].as_str().unwrap();
    assert_eq!(finished(&a, &h, id).await["status"], "failed");
    assert!(!a.playback.scratch.join(id).exists());
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn job_disk_idle_and_shutdown_cleanup() {
    let (a, h, root) = fixture().await;
    let s = settings::Settings {
        hardware_acceleration: "none".into(),
        max_concurrent_transcodes: 1,
        scratch_limit_gb: 0.1,
        ..Default::default()
    };
    sqlx::query("INSERT INTO server_settings VALUES(1,?)")
        .bind(serde_json::to_string(&s).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    let first = start(&a, &h, 0., "transcode").await;
    let id = first["id"].as_str().unwrap();
    assert!(create(State(a.clone()),h.clone(),Json(json!({"item_id":"i","start_seconds":0,"mode":"transcode","direct_play_supported":false}))).await.is_err());
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    assert!(!a.playback.scratch.join(id).exists());
    let reserved = a.playback.scratch.join("reserved");
    let f = tokio::fs::File::create(&reserved).await.unwrap();
    f.set_len(110 * 1024 * 1024).await.unwrap();
    drop(f);
    let full = start(&a, &h, 0., "transcode").await;
    let id = full["id"].as_str().unwrap();
    let result = finished(&a, &h, id).await;
    assert_eq!(result["status"], "failed");
    assert!(result["error"].as_str().unwrap().contains("Scratch"));
    assert!(!a.playback.scratch.join(id).exists());
    tokio::fs::remove_file(reserved).await.unwrap();
    let direct = start(&a, &h, 0., "transcode").await;
    let id = direct["id"].as_str().unwrap().to_owned();
    assert_eq!(finished(&a, &h, &id).await["status"], "ready");
    a.playback
        .sessions
        .lock()
        .await
        .get_mut(&id)
        .unwrap()
        .touched = std::time::Instant::now() - std::time::Duration::from_secs(4000);
    let reaper = tokio::spawn(reap(a.clone()));
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
    reaper.abort();
    assert!(!a.playback.sessions.lock().await.contains_key(&id));
    assert!(!a.playback.scratch.join(&id).exists());
    a.playback.shutdown().await;
    assert!(!a.playback.scratch.exists());
    tokio::fs::remove_dir_all(root).await.unwrap();
}
#[tokio::test]
async fn missing_ffmpeg_fails_and_cleans_scratch() {
    let (mut a, h, root) = fixture().await;
    Arc::get_mut(&mut a.playback).unwrap().ffmpeg = "/nonexistent/oxevu-ffmpeg".into();
    plan(&a, "none", 0, false).await;
    let s = start(&a, &h, 0., "transcode").await;
    let id = s["id"].as_str().unwrap();
    assert_eq!(finished(&a, &h, id).await["status"], "failed");
    assert!(!a.playback.scratch.join(id).exists());
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn hdr_is_rejected_and_stale_scratch_is_removed() {
    let (a, h, root) = fixture().await;
    plan(&a, "none", 0, false).await;
    let hdr = root.join("hdr.mkv");
    let status = tokio::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(root.join("source.mkv"))
        .args(["-c", "copy", "-color_trc", "smpte2084"])
        .arg(&hdr)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    sqlx::query("UPDATE items SET path=? WHERE id='i'")
        .bind(hdr.to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    let s = start(&a, &h, 0., "transcode").await;
    let out = finished(&a, &h, s["id"].as_str().unwrap()).await;
    assert_eq!(out["status"], "failed", "{out}");
    assert!(out["error"].as_str().unwrap().contains("HDR"));
    let data = a
        .playback
        .scratch
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let scratch = a.playback.scratch.clone();
    a.playback.shutdown().await;
    tokio::fs::create_dir_all(scratch.join("stale"))
        .await
        .unwrap();
    tokio::fs::write(scratch.join("stale/segment.ts"), b"stale")
        .await
        .unwrap();
    let restarted = Runtime::new(&data, &a.pool).await.unwrap();
    assert!(!restarted.scratch.join("stale").exists());
    restarted.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn scratch_overlap_and_concurrent_owner_are_rejected_without_media_writes() {
    let (a, _, root) = fixture().await;
    assert!(Runtime::new(&root, &a.pool).await.is_err());
    assert!(!root.join("transcodes").exists());
    let data = a.playback.scratch.parent().unwrap().parent().unwrap();
    let sentinel = a.playback.scratch.join("sentinel");
    tokio::fs::write(&sentinel, b"keep").await.unwrap();
    assert!(Runtime::new(data, &a.pool).await.is_err());
    assert!(sentinel.exists());
    let cap = runtime::capabilities(&a, false).await;
    assert!(cap["ffmpeg_path"].is_null());
    assert!(cap["scratch_path"].is_null());
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn stopping_active_encoder_kills_and_reaps_child() {
    let (a, h, root) = fixture().await;
    plan(&a, "none", 0, false).await;
    let long = root.join("long.mkv");
    let status = tokio::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-stream_loop", "39", "-i"])
        .arg(root.join("source.mkv"))
        .args(["-c", "copy"])
        .arg(&long)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    sqlx::query("UPDATE items SET path=?,data=? WHERE id='i'")
        .bind(long.to_string_lossy().as_ref())
        .bind(
            json!({"id":"i","library_id":"l","duration_seconds":320,"video_codec":"h264"})
                .to_string(),
        )
        .execute(&a.pool)
        .await
        .unwrap();
    let s = start(&a, &h, 0., "transcode").await;
    let id = s["id"].as_str().unwrap();
    let pid = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if let Some(pid) = a.playback.sessions.lock().await.get(id).and_then(|e| e.pid) {
                break pid;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    stop(State(a.clone()), h, Path(id.into())).await.unwrap();
    // SAFETY: signal zero only checks whether the already reaped child still exists.
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    assert!(!a.playback.scratch.join(id).exists());
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[path = "hardware_tests.rs"]
mod hardware_tests;
