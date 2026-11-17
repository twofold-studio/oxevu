use super::*;

pub(super) async fn stage(a: &App, title: &str, status: &str) -> Result<&'static str> {
    if status != "approved" {
        return Ok(match status {
            "fulfilled" => "Available",
            "declined" => "Declined",
            "cancelled" => "Cancelled",
            _ => "Awaiting approval",
        });
    }
    let failed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM acquisition_imports WHERE title_id=? AND state='failed')",
    )
    .bind(title)
    .fetch_one(&a.pool)
    .await
    .map_err(db)?;
    if failed {
        return Ok("Import needs review");
    }
    let jobs:Vec<String>=sqlx::query_scalar("SELECT state FROM acquisition_downloads WHERE title_id=? AND state NOT IN ('removed','duplicate')").bind(title).fetch_all(&a.pool).await.map_err(db)?;
    for (states, label) in [
        (&["checking", "downloading"][..], "Downloading"),
        (&["queued"][..], "Queued"),
        (&["error"][..], "Download needs attention"),
        (
            &["seeding", "completed"][..],
            "Downloaded; checking library availability",
        ),
        (&["paused"][..], "Transfer paused"),
    ] {
        if jobs.iter().any(|state| states.contains(&state.as_str())) {
            return Ok(label);
        }
    }
    let monitor = sqlx::query(
        "SELECT root,enabled,automatic,failures FROM acquisition_monitors WHERE title_id=?",
    )
    .bind(title)
    .fetch_optional(&a.pool)
    .await
    .map_err(db)?;
    let Some(monitor) = monitor else {
        return Ok("Choose a destination in Wanted");
    };
    if monitor.get::<Option<String>, _>("root").is_none() {
        return Ok("Choose a destination in Wanted");
    }
    if !monitor.get::<bool, _>("enabled") {
        return Ok("Monitoring paused");
    }
    if !config::load(a).await?.enabled {
        return Ok("Downloads disabled");
    }
    if monitor.get::<i64, _>("failures") > 0 {
        return Ok("Search delayed; administrator review needed");
    }
    Ok(if monitor.get::<bool, _>("automatic") {
        "Waiting for a suitable release"
    } else {
        "Awaiting release selection"
    })
}
