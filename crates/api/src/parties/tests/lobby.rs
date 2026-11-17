use super::*;

#[tokio::test]
async fn empty_party_is_discoverable_and_any_member_can_start_and_control_playback() {
    let (a, host, guest) = fixture().await;
    let (status, created) = request(&a, &host, "POST", "/parties", json!({"name":"Friday"})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(created["party"]["item_id"].is_null());
    let id = created["party"]["id"].as_str().unwrap();
    let path = format!("/parties/{id}");
    let control = format!("{path}/control");
    let listed = request(&a, &guest, "GET", "/parties", Value::Null).await.1;
    assert_eq!(listed["parties"][0]["id"], id);
    // Seeing a party never gives a nonmember control.
    assert_eq!(
        request(
            &a,
            &guest,
            "POST",
            &control,
            json!({"action":"item","item_id":"one"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&a, &guest, "POST", &format!("{path}/join"), Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&a, &guest, "POST", &control, json!({"action":"play"}))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let started = request(
        &a,
        &guest,
        "POST",
        &control,
        json!({"action":"item","item_id":"one","position_seconds":20}),
    )
    .await
    .1;
    assert_eq!(started["party"]["item_id"], "one");
    assert_eq!(started["party"]["paused"], false);
    assert_eq!(started["party"]["playback_revision"], 1);
    let paused = request(&a, &guest, "POST", &control, json!({"action":"pause"}))
        .await
        .1;
    assert_eq!(paused["party"]["paused"], true);
    assert_eq!(paused["party"]["playback_revision"], 1);
    let sought = request(
        &a,
        &host,
        "POST",
        &control,
        json!({"action":"seek","position_seconds":80}),
    )
    .await
    .1;
    assert_eq!(sought["party"]["position_seconds"], 80.);
    // Reopening the same title is a new navigation event, unlike pause/seek.
    let reopened = request(
        &a,
        &guest,
        "POST",
        &control,
        json!({"action":"item","item_id":"one"}),
    )
    .await
    .1;
    assert_eq!(reopened["party"]["playback_revision"], 2);
    assert_eq!(
        request(
            &a,
            &guest,
            "POST",
            &control,
            json!({"action":"item","item_id":"two"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    a.playback.shutdown().await;
}

#[tokio::test]
async fn empty_parties_still_require_playback_and_syncplay_permission() {
    let (a, host, guest) = fixture().await;
    request(&a, &host, "POST", "/parties", json!({"name":"Friday"})).await;
    sqlx::query("UPDATE user_access SET can_play=0 WHERE user_id='guest'")
        .execute(&a.pool)
        .await
        .unwrap();
    let listed = request(&a, &guest, "GET", "/parties", Value::Null).await.1;
    assert!(listed["parties"].as_array().unwrap().is_empty());
    a.playback.shutdown().await;
}
