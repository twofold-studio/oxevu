use super::*;

#[test]
fn xml_fragments_entities_and_cdata_preserve_complete_fields() {
    let xml = br#"<rss><channel><item><description>Ignored &amp; separate</description><title>Fast &amp; Furious &#38; <![CDATA[Friends &amp;]]></title><guid>magnet:?xt=urn:btih:123&amp;dn=Film&amp;tr=udp://tracker</guid><link>http://example.test/?id=1&amp;file=Film</link></item></channel></rss>"#;
    let parsed = torznab::parse(xml, false).unwrap();
    assert_eq!(parsed[0]["title"], "Fast & Furious & Friends &amp;");
    assert_eq!(
        parsed[0]["guid"],
        "magnet:?xt=urn:btih:123&dn=Film&tr=udp://tracker"
    );
    assert_eq!(parsed[0]["link"], "http://example.test/?id=1&file=Film");
    assert!(torznab::parse(b"<item><title>&unknown;</title></item>", false).is_err());
    assert!(torznab::parse(&vec![b' '; 2 * 1024 * 1024 + 1], false).is_err());
    let many = format!("<rss>{}</rss>", "<item><title>A</title></item>".repeat(101));
    assert_eq!(
        torznab::parse(many.as_bytes(), false)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        100
    );
    let large = format!(
        "<item><title>{}&amp;{}</title></item>",
        "x".repeat(4096),
        "x".repeat(4096)
    );
    assert!(torznab::parse(large.as_bytes(), false).is_err());
}

#[test]
fn release_ids_prefer_hashes_and_group_sources_without_duplicate_keys() {
    let first = json!({"guid":"same","title":"Movie","infohash":"a".repeat(40)});
    let second = json!({"guid":"same","title":"Movie","infohash":"b".repeat(40)});
    assert_ne!(
        release_identity::key("one", &first),
        release_identity::key("one", &second)
    );
    assert_eq!(
        release_identity::key("one", &first),
        release_identity::key("one", &first)
    );
    let a = json!({"id":"a","infohash":"a".repeat(40),"indexer_id":"one","indexer_name":"One","accepted":true,"seeders":5});
    let b = json!({"id":"b","infohash":"a".repeat(40),"indexer_id":"two","indexer_name":"Two","accepted":true,"seeders":4});
    let grouped = release_identity::group(vec![a.clone(), b, a]);
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0]["also_on"][0]["indexer_name"], "Two");
}

#[test]
fn size_limits_use_profile_values_and_zero_is_unlimited() {
    let mut profile = profiles::Profile {
        max_size_mb: 0,
        ..Default::default()
    };
    let release = json!({"height":1080,"size":30u64*1024*1024*1024,"seeders":3,"codec":"h264"});
    assert!(profiles::evaluate(&profile, &release).is_empty());
    profile.max_size_mb = 10 * 1024;
    assert!(
        profiles::evaluate(&profile, &release)
            .iter()
            .any(|reason| reason.contains("30.0 GiB") && reason.contains("10.0 GiB"))
    );
    profile.min_size_mb = 1024;
    profile.max_size_mb = 0;
    assert!(
        profiles::evaluate(
            &profile,
            &json!({"height":1080,"size":1,"seeders":3,"codec":"h264"})
        )
        .iter()
        .any(|reason| reason.contains("minimum"))
    );
}

#[tokio::test]
async fn full_search_stores_distinct_magnets_and_reports_named_retryable_failures() {
    let router = Router::new().route("/{source}", axum::routing::get(|Path(source):Path<String>| async move {
        if source == "busy" { return (StatusCode::TOO_MANY_REQUESTS, String::new()); }
        let (start,count) = if source == "first" { (0,57) } else { (57,58) };
        let items: String = (start..start+count).map(|n| format!("<item><title>Generated fixture 1080p WEB-DL x264</title><guid>magnet:?xt=urn:btih:{n:040x}&amp;dn=Film&amp;tr=udp://same-tracker</guid><link>magnet:?xt=urn:btih:{n:040x}&amp;dn=Film</link><enclosure url='magnet:?xt=urn:btih:{n:040x}' length='1000000'/><attr name='seeders' value='5'/></item>")).collect();
        (StatusCode::OK, format!("<rss>{items}</rss>"))
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let a = crate::tests::app().await;
    let admin = crate::tests::account(&a, "admin", true).await;
    let title = title(&a, "movie", 100).await;
    for name in ["first", "second", "busy"] {
        let _ = indexers::put(
            State(a.clone()),
            admin.clone(),
            Path(name.into()),
            Json(indexers::Indexer {
                name: name.into(),
                endpoint: format!("{base}/{name}"),
                enabled: true,
                capabilities: json!({"search":true}),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    }
    let first = releases::search_for(
        &a,
        title["id"].as_str().unwrap(),
        &releases::Search::default(),
    )
    .await
    .unwrap();
    assert_eq!(first["releases"].as_array().unwrap().len(), 115);
    let unique: std::collections::HashSet<_> = first["releases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect();
    assert_eq!(unique.len(), 115);
    assert_eq!(first["errors"][0]["indexer_name"], "busy");
    assert_eq!(first["errors"][0]["retryable"], true);
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM acquisition_releases")
        .fetch_one(&a.pool)
        .await
        .unwrap();
    assert_eq!(stored, 115);
    let second = releases::search_for(
        &a,
        title["id"].as_str().unwrap(),
        &releases::Search::default(),
    )
    .await
    .unwrap();
    assert_eq!(first["releases"], second["releases"]);
    a.playback.shutdown().await;
    task.abort();
    let _ = task.await;
}
