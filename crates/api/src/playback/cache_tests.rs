use super::*;

#[tokio::test]
async fn cached_output_is_reused_without_encoder_and_still_checks_access() {
    let (a, h, root) = fixture().await;
    plan(&a, "none", 0, true).await;
    let first = start(&a, &h, 0., "transcode").await;
    let id = first["id"].as_str().unwrap();
    assert_eq!(finished(&a, &h, id).await["status"], "ready");
    stop(State(a.clone()), h.clone(), Path(id.into()))
        .await
        .unwrap();
    let reused = start(&a, &h, 3., "transcode").await;
    let cached = reused["id"].as_str().unwrap();
    assert_eq!(reused["cache_hit"], true);
    assert_eq!(reused["cached_ranges"][0]["start"], 0.);
    assert!(reused["cached_ranges"][0]["end"].as_f64().unwrap() >= 7.);
    assert_eq!(reused["status"], "ready");
    assert!(reused["start_seconds"].as_f64().unwrap() <= 3.);
    assert!(
        a.playback
            .sessions
            .lock()
            .await
            .get(cached)
            .unwrap()
            .pid
            .is_none()
    );
    readable(&a, &h, cached).await;
    let viewer = crate::tests::account(&a, "viewer", false).await;
    assert!(
        get(State(a.clone()), viewer, Path(cached.into()))
            .await
            .is_err()
    );
    stop(State(a.clone()), h.clone(), Path(cached.into()))
        .await
        .unwrap();
    let settings = settings::Settings {
        hardware_acceleration: "none".into(),
        max_height: 144,
        ..Default::default()
    };
    sqlx::query("UPDATE server_settings SET data=? WHERE id=1")
        .bind(serde_json::to_string(&settings).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    let changed = start(&a, &h, 0., "transcode").await;
    assert_eq!(changed["cache_hit"], false);
    assert_eq!(changed["cached_ranges"], json!([]));
    stop(
        State(a.clone()),
        h,
        Path(changed["id"].as_str().unwrap().into()),
    )
    .await
    .unwrap();
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}
