use super::*;

#[tokio::test]
#[ignore = "requires NVIDIA GPU0"]
async fn cuda_hevc10_sessions_and_failure_policy() {
    let (a, h, root) = fixture().await;
    let source = root.join("source.mkv");
    assert!(
        tokio::process::Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=640x360:rate=24",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440",
                "-t",
                "6",
                "-c:v",
                "libx265",
                "-pix_fmt",
                "yuv420p10le",
                "-x265-params",
                "pools=1:frame-threads=1:log-level=error",
                "-c:a",
                "aac"
            ])
            .arg(&source)
            .status()
            .await
            .unwrap()
            .success()
    );
    let metadata = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,pix_fmt",
            "-of",
            "json",
        ])
        .arg(&source)
        .output()
        .await
        .unwrap();
    let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
    assert_eq!(metadata["streams"][0]["codec_name"], "hevc");
    assert_eq!(metadata["streams"][0]["pix_fmt"], "yuv420p10le");
    eprintln!("HEVC10 fixture: {metadata}");
    for (encoding, gpu, fallback, ready) in [
        (true, 0, false, true),
        (false, 0, false, true),
        (true, 31, true, true),
        (true, 31, false, false),
    ] {
        let settings = settings::Settings {
            hardware_decoding: true,
            hardware_encoding: encoding,
            gpu_index: gpu,
            allow_cpu_fallback: fallback,
            max_height: 180,
            ..Default::default()
        };
        let _ = settings::put(State(a.clone()), h.clone(), Json(settings))
            .await
            .unwrap();
        let s = start(&a, &h, 0., "transcode").await;
        let id = s["id"].as_str().unwrap();
        let out = finished(&a, &h, id).await;
        eprintln!("hardware acceptance {out}");
        assert_eq!(
            out["status"],
            if ready { "ready" } else { "failed" },
            "{out}"
        );
        if ready {
            assert_eq!(out["decoder"], if gpu == 0 { "cuda" } else { "cpu" });
            readable(&a, &h, id).await;
        }
        stop(State(a.clone()), h.clone(), Path(id.into()))
            .await
            .unwrap();
        assert!(!a.playback.scratch.join(id).exists());
    }
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
#[ignore = "requires NVIDIA GPU0 and OXEVU_TEST_ARCANE source"]
async fn cuda_real_arcane_excerpt() {
    let input = std::env::var("OXEVU_TEST_ARCANE").expect("set OXEVU_TEST_ARCANE");
    let (a, h, root) = fixture().await;
    let source = root.join("source.mkv");
    assert!(
        tokio::process::Command::new("ffmpeg")
            .args(["-nostdin", "-v", "error", "-y", "-ss", "60", "-i"])
            .arg(input)
            .args(["-t", "12", "-map", "0:v:0", "-map", "0:a:0", "-c", "copy"])
            .arg(&source)
            .status()
            .await
            .unwrap()
            .success()
    );
    let settings = settings::Settings {
        hardware_decoding: true,
        allow_cpu_fallback: false,
        max_height: 720,
        ..Default::default()
    };
    let input = super::source::Input::local(source);
    let cmd = worker::command(&a, &input, &root, 0., "nvenc", &settings, true);
    let args: Vec<_> = cmd
        .as_std()
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    eprintln!("Arcane FFmpeg args: {args:?}");
    assert!(args.windows(2).any(|p| p == ["-hwaccel", "cuda"]));
    assert!(args.iter().any(|a| a.contains("scale_cuda=")));
    let _ = settings::put(State(a.clone()), h.clone(), Json(settings))
        .await
        .unwrap();
    assert_eq!(
        runtime::capabilities(&a, true).await["hardware_decode_available"],
        true
    );
    let s = start(&a, &h, 0., "transcode").await;
    let id = s["id"].as_str().unwrap();
    let out = finished(&a, &h, id).await;
    eprintln!("Arcane acceptance: {out}");
    assert_eq!(out["status"], "ready", "{out}");
    assert_eq!(out["decoder"], "cuda");
    readable(&a, &h, id).await;
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-count_frames",
            "-show_entries",
            "stream=nb_read_frames,height",
            "-of",
            "json",
        ])
        .arg(a.playback.scratch.join(id).join("index.m3u8"))
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    let frames: Value = serde_json::from_slice(&output.stdout).unwrap();
    eprintln!("Arcane output frames: {frames}");
    assert!(
        frames["streams"][0]["nb_read_frames"]
            .as_str()
            .unwrap()
            .parse::<u32>()
            .unwrap()
            > 100
    );
    assert!(frames["streams"][0]["height"].as_u64().unwrap() <= 720);
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    assert!(!a.playback.scratch.join(id).exists());
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}
