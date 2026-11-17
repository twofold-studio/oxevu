use super::*;

#[tokio::test]
#[ignore = "requires GPU0 and OXEVU_TEST_ARCANE; measures real HLS startup"]
async fn real_arcane_first_manifest() {
    let input = PathBuf::from(std::env::var("OXEVU_TEST_ARCANE").unwrap());
    let input = tokio::fs::canonicalize(input).await.unwrap();
    let a = crate::tests::app().await;
    let h = crate::tests::account(&a, "owner", true).await;
    sqlx::query("INSERT INTO libraries VALUES('l','Benchmark',?,'movies')")
        .bind(input.parent().unwrap().to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l',?,?)")
        .bind(input.to_string_lossy().as_ref())
        .bind(
            json!({"id":"i","library_id":"l","duration_seconds":2500,"video_codec":"hevc"})
                .to_string(),
        )
        .execute(&a.pool)
        .await
        .unwrap();
    let settings = settings::Settings {
        hardware_decoding: true,
        allow_cpu_fallback: false,
        max_height: 720,
        max_video_bitrate_mbps: 4.,
        ..Default::default()
    };
    sqlx::query("INSERT INTO server_settings VALUES(1,?)")
        .bind(serde_json::to_string(&settings).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    for round in 0..2 {
        for offset in [0., 600., 1200.] {
            let clock = std::time::Instant::now();
            let session = start(&a, &h, offset, "transcode").await;
            let id = session["id"].as_str().unwrap();
            let playlist = a.playback.scratch.join(id).join("index.m3u8");
            tokio::time::timeout(std::time::Duration::from_secs(20), async {
                loop {
                    if tokio::fs::try_exists(&playlist).await.unwrap() {
                        break;
                    }
                    let state = get(State(a.clone()), h.clone(), Path(id.into()))
                        .await
                        .unwrap()
                        .0;
                    assert_ne!(state["session"]["status"], "failed", "{state}");
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            eprintln!(
                "seek-startup round={round} offset={offset} manifest_ms={}",
                clock.elapsed().as_millis()
            );
            let manifest = tokio::fs::read_to_string(&playlist).await.unwrap();
            assert!(manifest.contains("#EXTINF:"));
            for line in manifest
                .lines()
                .filter_map(|line| line.strip_prefix("#EXTINF:"))
            {
                let seconds: f64 = line.split(',').next().unwrap().parse().unwrap();
                assert!(
                    seconds <= 2.1,
                    "NVIDIA must produce actual two-second chunks: {seconds}"
                );
            }
            stop(State(a.clone()), h.clone(), Path(id.into()))
                .await
                .unwrap();
            assert!(!a.playback.scratch.join(id).exists());
        }
    }
    a.playback.shutdown().await;
}
