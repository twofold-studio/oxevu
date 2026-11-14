use super::*;
#[tokio::test]
async fn changed_item_requires_every_members_access_and_listing_hides_private_rooms() {
    let (a, host, guest) = fixture().await;
    let shared = create_room(&a, &host, "one").await;
    let private = create_room(&a, &host, "two").await;
    let listed = request(&a, &guest, "GET", "/parties", Value::Null).await.1;
    assert_eq!(listed["parties"].as_array().unwrap().len(), 1);
    assert_eq!(listed["parties"][0]["id"], shared["id"]);
    let private_path = format!("/parties/{}/join", private["id"].as_str().unwrap());
    assert_eq!(
        request(&a, &guest, "POST", &private_path, Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let path = format!("/parties/{}", shared["id"].as_str().unwrap());
    assert_eq!(
        request(&a, &guest, "POST", &format!("{path}/join"), Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &a,
            &host,
            "POST",
            &format!("{path}/control"),
            json!({"action":"item","item_id":"two"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&a, &guest, "GET", &path, Value::Null).await.1["party"]["item_id"],
        "one"
    );
    sqlx::query("INSERT INTO library_grants VALUES('guest','private')")
        .execute(&a.pool)
        .await
        .unwrap();
    let changed = request(
        &a,
        &host,
        "POST",
        &format!("{path}/control"),
        json!({"action":"item","item_id":"two"}),
    )
    .await
    .1["party"]
        .clone();
    assert_eq!(changed["item_id"], "two");
    assert_eq!(changed["paused"], true);
    assert_eq!(changed["position_seconds"], 0.);
    a.playback.shutdown().await;
}
#[tokio::test]
async fn revocation_disabled_users_and_idle_rooms_are_pruned() {
    let (a, host, guest) = fixture().await;
    let room = create_room(&a, &host, "one").await;
    let id = room["id"].as_str().unwrap();
    let path = format!("/parties/{id}");
    request(&a, &guest, "POST", &format!("{path}/join"), Value::Null).await;
    sqlx::query("DELETE FROM sessions WHERE user_id='guest'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&a, &guest, "GET", &path, Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
    let polled = request(&a, &host, "GET", &path, Value::Null).await.1;
    assert_eq!(polled["party"]["members"].as_array().unwrap().len(), 1);
    sqlx::query("UPDATE user_access SET disabled=1 WHERE user_id='host'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&a, &host, "GET", &path, Value::Null).await.0,
        StatusCode::FORBIDDEN
    );
    {
        let mut rooms = a.parties.rooms.lock().await;
        prune(&a, &mut rooms).await.unwrap();
        assert!(rooms.is_empty());
    }
    sqlx::query("UPDATE user_access SET disabled=0 WHERE user_id='host'")
        .execute(&a.pool)
        .await
        .unwrap();
    let room = create_room(&a, &host, "one").await;
    let id = room["id"].as_str().unwrap();
    {
        a.parties.rooms.lock().await.get_mut(id).unwrap().touched =
            std::time::Instant::now() - std::time::Duration::from_secs(600);
    }
    assert!(
        request(&a, &host, "GET", "/parties", Value::Null).await.1["parties"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    a.playback.shutdown().await;
}
#[tokio::test]
async fn playback_syncplay_and_library_revocation_remove_members() {
    let (a, host, guest) = fixture().await;
    let created = create_room(&a, &host, "one").await;
    let path = format!("/parties/{}", created["id"].as_str().unwrap());
    for revoke in [
        "UPDATE user_access SET can_play=0 WHERE user_id='guest'",
        "UPDATE user_access SET syncplay_access='disabled' WHERE user_id='guest'",
        "DELETE FROM library_grants WHERE user_id='guest'",
    ] {
        assert_eq!(
            request(&a, &guest, "POST", &format!("{path}/join"), Value::Null)
                .await
                .0,
            StatusCode::OK
        );
        sqlx::query(revoke).execute(&a.pool).await.unwrap();
        assert_eq!(
            request(&a, &guest, "GET", &path, Value::Null).await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(&a, &guest, "POST", &format!("{path}/join"), Value::Null)
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert!(
            request(&a, &guest, "GET", "/parties", Value::Null).await.1["parties"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        sqlx::query(
            "UPDATE user_access SET can_play=1,syncplay_access='join' WHERE user_id='guest'",
        )
        .execute(&a.pool)
        .await
        .unwrap();
        sqlx::query("INSERT OR IGNORE INTO library_grants VALUES('guest','shared')")
            .execute(&a.pool)
            .await
            .unwrap();
    }
    a.playback.shutdown().await;
}
