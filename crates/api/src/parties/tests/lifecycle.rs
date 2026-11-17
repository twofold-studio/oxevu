use super::*;
#[tokio::test]
async fn two_user_authenticated_lifecycle_and_shared_timeline() {
    let (a, host, guest) = fixture().await;
    assert_eq!(
        request(&a, &HeaderMap::new(), "GET", "/parties", Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &a,
            &guest,
            "POST",
            "/parties",
            json!({"name":"Guest","item_id":"one"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let created = create_room(&a, &host, "one").await;
    let id = created["id"].as_str().unwrap();
    let path = format!("/parties/{id}");
    assert_eq!(
        request(&a, &guest, "GET", &path, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    let joined = request(&a, &guest, "POST", &format!("{path}/join"), Value::Null)
        .await
        .1["party"]
        .clone();
    assert_eq!(joined["members"].as_array().unwrap().len(), 2);
    assert!(joined["revision"].as_u64() > created["revision"].as_u64());
    assert_eq!(
        request(
            &a,
            &guest,
            "POST",
            &format!("{path}/control"),
            json!({"action":"play"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let playing = request(
        &a,
        &host,
        "POST",
        &format!("{path}/control"),
        json!({"action":"play","position_seconds":10}),
    )
    .await
    .1["party"]
        .clone();
    assert_eq!(playing["paused"], false);
    {
        let mut rooms = a.parties.rooms.lock().await;
        let r = rooms.get_mut(id).unwrap();
        r.updated = r.updated.saturating_sub(2000);
    }
    let paused = request(
        &a,
        &host,
        "POST",
        &format!("{path}/control"),
        json!({"action":"pause"}),
    )
    .await
    .1["party"]
        .clone();
    assert_eq!(paused["paused"], true);
    let p = paused["position_seconds"].as_f64().unwrap();
    assert!((12. ..12.5).contains(&p), "{paused}");
    let polled = request(&a, &guest, "GET", &path, Value::Null).await.1["party"].clone();
    assert_eq!(polled["position_seconds"], paused["position_seconds"]);
    assert_eq!(polled["revision"], paused["revision"]);
    let sought = request(
        &a,
        &host,
        "POST",
        &format!("{path}/control"),
        json!({"action":"seek","position_seconds":100}),
    )
    .await
    .1["party"]
        .clone();
    assert_eq!(sought["position_seconds"], 100.);
    assert_eq!(sought["paused"], true);
    assert_eq!(
        request(&a, &guest, "POST", &format!("{path}/leave"), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&a, &guest, "GET", &path, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&a, &host, "POST", &format!("{path}/leave"), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&a, &host, "GET", &path, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    a.playback.shutdown().await;
}
#[test]
fn deterministic_timeline_clamps_elapsed_and_checks_finite_positions() {
    let room = Room {
        id: "r".into(),
        name: "r".into(),
        host: "h".into(),
        item: Some("i".into()),
        paused: false,
        position: 10.,
        updated: 1000,
        revision: 1,
        playback_revision: 1,
        duration: Some(20.),
        members: Default::default(),
        touched: std::time::Instant::now(),
    };
    assert_eq!(room.current(500), 10.);
    assert_eq!(room.current(2500), 11.5);
    assert_eq!(room.current(50000), 20.);
    assert!(position(Some(&json!(-1)), None).is_err());
    assert!(position(Some(&json!(604801)), None).is_err());
}

#[tokio::test]
async fn party_members_playback_sessions_keep_independent_ownership() {
    let (a, host, guest) = fixture().await;
    let created = create_room(&a, &host, "one").await;
    request(
        &a,
        &guest,
        "POST",
        &format!("/parties/{}/join", created["id"].as_str().unwrap()),
        Value::Null,
    )
    .await;
    let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir(&root).await.unwrap();
    let source = root.join("sample.mp4");
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=320x180:rate=1",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-threads",
            "1",
        ])
        .arg(&source)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    sqlx::query("UPDATE libraries SET path=? WHERE id='shared'")
        .bind(root.to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE items SET path=? WHERE id='one'")
        .bind(source.to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    let body =
        json!({"item_id":"one","start_seconds":0,"mode":"direct","direct_play_supported":true});
    let first = playback::create(State(a.clone()), host.clone(), Json(body.clone()))
        .await
        .unwrap()
        .0;
    let second = playback::create(State(a.clone()), guest.clone(), Json(body))
        .await
        .unwrap()
        .0;
    assert_ne!(first["session"]["id"], second["session"]["id"]);
    let id = first["session"]["id"].as_str().unwrap();
    assert!(
        playback::heartbeat(State(a.clone()), guest, Path(id.into()))
            .await
            .is_err()
    );
    assert!(
        playback::heartbeat(State(a.clone()), host, Path(id.into()))
            .await
            .is_ok()
    );
    a.playback.shutdown().await;
    tokio::fs::remove_dir_all(root).await.unwrap();
}
