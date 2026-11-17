use super::*;
use scope::{Episode, Scope};
mod adoption;
mod automation;
mod download_limits;
mod fixture;
mod flow;
mod magnet_flow;
mod publication;
mod release_search;
mod series;
mod torrent_redirects;

pub(super) async fn title(a: &App, kind: &str, id: u64) -> Value {
    let title = json!({"id":format!("tmdb:{kind}:{id}"),"provider_id":id,"kind":kind,"title":"Generated fixture"});
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES(?,?,?,?)")
        .bind(title["id"].as_str())
        .bind(kind)
        .bind(id as i64)
        .bind(title.to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    if kind == "tv" {
        for season in 1..=2 {
            for episode in 1..=2 {
                sqlx::query("INSERT INTO acquisition_episodes VALUES(?,?,?,'{}','2020-01-01')")
                    .bind(title["id"].as_str())
                    .bind(season)
                    .bind(episode)
                    .execute(&a.pool)
                    .await
                    .unwrap();
            }
        }
    }
    title
}

#[tokio::test]
async fn overlapping_requests_preserve_ownership_approval_and_deduplicate() {
    let a = crate::tests::app().await;
    let admin = crate::tests::account(&a, "admin", true).await;
    let first = crate::tests::account(&a, "first", false).await;
    let second = crate::tests::account(&a, "second", false).await;
    let t = title(&a, "tv", 100).await;
    let u = user(&a, &first).await.unwrap();
    let one = requests::create_for(
        &a,
        &u,
        &t,
        Scope {
            mode: "seasons".into(),
            seasons: vec![1, 1],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let duplicate = requests::create_for(
        &a,
        &u,
        &t,
        Scope {
            mode: "seasons".into(),
            seasons: vec![1],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(one, duplicate);
    let u = user(&a, &second).await.unwrap();
    let two = requests::create_for(&a, &u, &t, Scope::default())
        .await
        .unwrap();
    assert_ne!(one, two);
    let result = requests::list(State(a.clone()), first.clone())
        .await
        .unwrap()
        .0;
    assert_eq!(result["requests"].as_array().unwrap().len(), 1);
    assert_eq!(result["requests"][0]["status"], "pending");
    let approve = || requests::Decision {
        status: "approved".into(),
        note: String::new(),
    };
    assert_eq!(
        requests::update(
            State(a.clone()),
            first.clone(),
            Path(two.clone()),
            Json(approve())
        )
        .await
        .unwrap_err()
        .0,
        StatusCode::FORBIDDEN
    );
    let _ = requests::update(
        State(a.clone()),
        admin.clone(),
        Path(two.clone()),
        Json(approve()),
    )
    .await
    .unwrap();
    let _ = requests::update(
        State(a.clone()),
        first.clone(),
        Path(one),
        Json(requests::Decision {
            status: "cancelled".into(),
            note: String::new(),
        }),
    )
    .await
    .unwrap();
    let other = requests::view(&a, &two, &admin).await.unwrap();
    assert_eq!(other["status"], "approved");
}

#[tokio::test]
async fn quota_permissions_and_unknown_episode_selection_are_enforced() {
    let a = crate::tests::app().await;
    let h = crate::tests::account(&a, "viewer", false).await;
    let u = user(&a, &h).await.unwrap();
    let t = title(&a, "movie", 1).await;
    sqlx::query("INSERT INTO acquisition_permissions(user_id,request_limit) VALUES(?,0)")
        .bind(u["id"].as_str())
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        requests::create_for(&a, &u, &t, Scope::default())
            .await
            .is_err()
    );
    sqlx::query("UPDATE acquisition_permissions SET request_limit=10,can_request=0")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        requests::create_for(&a, &u, &t, Scope::default())
            .await
            .is_err()
    );
    sqlx::query("UPDATE acquisition_permissions SET can_request=1")
        .execute(&a.pool)
        .await
        .unwrap();
    let t = title(&a, "tv", 2).await;
    assert!(
        requests::create_for(
            &a,
            &u,
            &t,
            Scope {
                mode: "episodes".into(),
                episodes: vec![Episode {
                    season: 7,
                    episode: 1
                }],
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        access::get(State(a), HeaderMap::new()).await.unwrap_err().0,
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn scopes_cover_only_the_selected_catalog_units() {
    let mut scope = Scope {
        mode: "episodes".into(),
        episodes: vec![Episode {
            season: 2,
            episode: 5,
        }],
        ..Default::default()
    };
    scope.normalize("tv", "2026-11-16").unwrap();
    assert!(scope.covers(2, 5, None));
    assert!(!scope.covers(1, 5, None));
    scope.mode = "future".into();
    scope.normalize("tv", "2026-11-16").unwrap();
    assert!(scope.covers(2, 6, Some("2026-11-17")));
    assert!(!scope.covers(2, 5, Some("2026-11-15")));
    assert!(!scope.covers(2, 7, None));
    assert!(!scope.covers(0, 1, Some("2026-11-17")));
}
