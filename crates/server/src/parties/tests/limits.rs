use super::*;
#[tokio::test]
async fn invalid_controls_do_not_change_revision_and_room_count_is_bounded() {
    let (a, host, _) = fixture().await;
    let room = create_room(&a, &host, "one").await;
    let path = format!("/parties/{}/control", room["id"].as_str().unwrap());
    for body in [
        json!({"action":"seek"}),
        json!({"action":"seek","position_seconds":-1}),
        json!({"action":"seek","position_seconds":1001}),
        json!({"action":"item","item_id":"missing"}),
        json!({"action":"invalid"}),
        json!({"action":"play","position_seconds":"NaN"}),
    ] {
        assert!(!request(&a, &host, "POST", &path, body).await.0.is_success());
    }
    assert_eq!(
        a.parties
            .rooms
            .lock()
            .await
            .values()
            .next()
            .unwrap()
            .revision,
        1
    );
    for _ in 1..MAX_ROOMS {
        create_room(&a, &host, "one").await;
    }
    assert_eq!(
        request(
            &a,
            &host,
            "POST",
            "/parties",
            json!({"name":"Overflow","item_id":"one"})
        )
        .await
        .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    a.playback.shutdown().await;
}
#[tokio::test]
async fn membership_limit_idempotent_join_and_member_idle_cleanup() {
    let (a, host, _) = fixture().await;
    let created = create_room(&a, &host, "one").await;
    let id = created["id"].as_str().unwrap();
    let path = format!("/parties/{id}");
    let mut members = Vec::new();
    for i in 0..MAX_MEMBERS {
        let name = format!("member{i}");
        let h = crate::tests::account(&a, &name, false).await;
        sqlx::query("INSERT INTO library_grants VALUES(?,'shared')")
            .bind(&name)
            .execute(&a.pool)
            .await
            .unwrap();
        let (status, _) = request(&a, &h, "POST", &format!("{path}/join"), Value::Null).await;
        assert_eq!(
            status,
            if i + 1 < MAX_MEMBERS {
                StatusCode::OK
            } else {
                StatusCode::TOO_MANY_REQUESTS
            }
        );
        members.push(h);
    }
    let before = request(&a, &host, "GET", &path, Value::Null).await.1;
    let after = request(
        &a,
        &members[0],
        "POST",
        &format!("{path}/join"),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(before["party"]["revision"], after["party"]["revision"]);
    a.parties
        .rooms
        .lock()
        .await
        .get_mut(id)
        .unwrap()
        .members
        .get_mut("member0")
        .unwrap()
        .touched = std::time::Instant::now() - std::time::Duration::from_secs(120);
    assert_eq!(
        request(&a, &members[0], "GET", &path, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    a.playback.shutdown().await;
}
