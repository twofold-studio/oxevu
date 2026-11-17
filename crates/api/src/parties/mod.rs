use crate::*;
mod model;
pub(crate) use model::Store;
use model::*;
fn missing() -> Error {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error":"party unavailable"})),
    )
}
#[cfg(test)]
pub(crate) fn routes() -> Router<App> {
    crate::contract::parties::router().split_for_parts().0
}
pub(crate) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    let mut rooms = a.parties.rooms.lock().await;
    prune(&a, &mut rooms).await?;
    let now = a.parties.now();
    let mut parties = Vec::new();
    for room in rooms.values() {
        if access(&a, u.clone(), room.item.as_deref()).await.is_ok() {
            parties.push(room.snapshot(now))
        }
    }
    Ok(Json(json!({"parties":parties})))
}
pub(crate) async fn create(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    if u["syncplay_access"] != "create_join" {
        return Err(users::forbidden());
    }
    let name = v["name"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 100 && !s.chars().any(char::is_control))
        .ok_or_else(|| bad("invalid party name"))?;
    let item = v
        .get("item_id")
        .filter(|value| !value.is_null())
        .map(|value| value.as_str().ok_or_else(|| bad("invalid item_id")))
        .transpose()?;
    access(&a, u.clone(), item).await?;
    let duration = match item {
        Some(item) => duration(&a, item).await?,
        None => None,
    };
    let mut rooms = a.parties.rooms.lock().await;
    prune(&a, &mut rooms).await?;
    if rooms.len() >= MAX_ROOMS {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"party room limit reached"})),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = a.parties.now();
    let mut room = Room {
        id: id.clone(),
        name: name.trim().into(),
        host: u["id"].as_str().unwrap_or_default().into(),
        item: item.map(str::to_owned),
        paused: true,
        position: 0.,
        updated: now,
        revision: 1,
        playback_revision: u64::from(item.is_some()),
        duration,
        members: Default::default(),
        touched: std::time::Instant::now(),
    };
    touch(&mut room, &u, &h);
    let value = room.snapshot(now);
    rooms.insert(id, room);
    Ok(Json(json!({"party":value})))
}
pub(crate) async fn join(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    let mut rooms = a.parties.rooms.lock().await;
    prune_one(&a, &mut rooms, &id).await?;
    let room = rooms.get_mut(&id).ok_or_else(missing)?;
    access(&a, u.clone(), room.item.as_deref()).await?;
    let member = room
        .members
        .contains_key(u["id"].as_str().unwrap_or_default());
    if !member && room.members.len() >= MAX_MEMBERS {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"party member limit reached"})),
        ));
    }
    touch(room, &u, &h);
    if !member {
        room.revision += 1;
    }
    Ok(Json(json!({"party":room.snapshot(a.parties.now())})))
}
pub(crate) async fn read(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    let mut rooms = a.parties.rooms.lock().await;
    prune_one(&a, &mut rooms, &id).await?;
    let room = rooms.get_mut(&id).ok_or_else(missing)?;
    if !room
        .members
        .contains_key(u["id"].as_str().unwrap_or_default())
    {
        return Err(missing());
    }
    access(&a, u.clone(), room.item.as_deref()).await?;
    touch(room, &u, &h);
    Ok(Json(json!({"party":room.snapshot(a.parties.now())})))
}
pub(crate) async fn leave(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let u = user(&a, &h).await?;
    let mut rooms = a.parties.rooms.lock().await;
    prune_one(&a, &mut rooms, &id).await?;
    let room = rooms.get_mut(&id).ok_or_else(missing)?;
    let user = u["id"].as_str().unwrap_or_default();
    if room.members.remove(user).is_none() {
        return Err(missing());
    }
    if room.host == user {
        rooms.remove(&id);
    } else {
        room.revision += 1;
    }
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn control(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    let mut rooms = a.parties.rooms.lock().await;
    prune_one(&a, &mut rooms, &id).await?;
    let room = rooms.get_mut(&id).ok_or_else(missing)?;
    if !room
        .members
        .contains_key(u["id"].as_str().unwrap_or_default())
    {
        return Err(users::forbidden());
    }
    access(&a, u.clone(), room.item.as_deref()).await?;
    let action = v["action"]
        .as_str()
        .filter(|s| matches!(*s, "play" | "pause" | "seek" | "item"))
        .ok_or_else(|| bad("invalid party action"))?;
    let mut item = room.item.clone();
    let mut duration = room.duration;
    if action == "item" {
        let selected = v["item_id"]
            .as_str()
            .ok_or_else(|| bad("item_id required"))?;
        for user in room.members.keys() {
            access(&a, users::profile(&a, user).await?, Some(selected)).await?;
        }
        duration = model::duration(&a, selected).await?;
        item = Some(selected.to_owned());
    } else if item.is_none() {
        return Err(bad("choose a title before controlling playback"));
    }
    let supplied = position(v.get("position_seconds"), duration)?;
    if action == "seek" && supplied.is_none() {
        return Err(bad("seek requires position_seconds"));
    }
    let now = a.parties.now();
    let position = supplied.unwrap_or_else(|| {
        if action == "item" {
            0.
        } else {
            room.current(now)
        }
    });
    room.item = item;
    room.duration = duration;
    room.position = position;
    room.updated = now;
    match action {
        "play" | "item" => room.paused = false,
        "pause" => room.paused = true,
        _ => {}
    }
    room.revision += 1;
    if action == "item" {
        room.playback_revision += 1;
    }
    touch(room, &u, &h);
    Ok(Json(json!({"party":room.snapshot(now)})))
}
pub(crate) async fn reap(a: App) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let mut rooms = a.parties.rooms.lock().await;
        if prune(&a, &mut rooms).await.is_err() {
            eprintln!("party cleanup failed");
        }
    }
}
#[cfg(test)]
mod tests;
