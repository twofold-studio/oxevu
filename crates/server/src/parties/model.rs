use super::*;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
pub(crate) const MAX_ROOMS: usize = 32;
pub(crate) const MAX_MEMBERS: usize = 16;
const MEMBER_IDLE: Duration = Duration::from_secs(90);
const ROOM_IDLE: Duration = Duration::from_secs(300);
pub(crate) struct Store {
    pub rooms: Mutex<BTreeMap<String, Room>>,
    epoch_ms: u64,
    started: Instant,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            rooms: Mutex::new(BTreeMap::new()),
            epoch_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64,
            started: Instant::now(),
        }
    }
}
impl Store {
    pub fn now(&self) -> u64 {
        self.epoch_ms
            .saturating_add(self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64)
    }
}
pub(crate) struct Member {
    pub username: String,
    pub token: String,
    pub touched: Instant,
}
pub(crate) struct Room {
    pub id: String,
    pub name: String,
    pub host: String,
    pub item: String,
    pub paused: bool,
    pub position: f64,
    pub updated: u64,
    pub revision: u64,
    pub duration: Option<f64>,
    pub members: BTreeMap<String, Member>,
    pub touched: Instant,
}
impl Room {
    pub fn current(&self, now: u64) -> f64 {
        let position = self.position
            + if self.paused {
                0.
            } else {
                now.saturating_sub(self.updated) as f64 / 1000.
            };
        self.duration
            .map_or(position, |duration| position.min(duration))
    }
    pub fn snapshot(&self, now: u64) -> Value {
        json!({"id":self.id,"name":self.name,"host_user_id":self.host,"item_id":self.item,"paused":self.paused,"position_seconds":self.position,"updated_at_ms":self.updated,"server_now_ms":now,"revision":self.revision,"members":self.members.iter().map(|(id,m)|json!({"user_id":id,"username":m.username})).collect::<Vec<_>>()})
    }
}
pub(crate) async fn access(a: &App, u: Value, item: &str) -> Result<Value> {
    if !matches!(u["syncplay_access"].as_str(), Some("create_join" | "join")) {
        return Err(users::forbidden());
    }
    users::authorize_profile(a, u, item, true).await
}
pub(crate) async fn duration(a: &App, item: &str) -> Result<Option<f64>> {
    let data: String = sqlx::query_scalar("SELECT data FROM items WHERE id=?")
        .bind(item)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let data: Value = serde_json::from_str(&data).map_err(bad)?;
    Ok(data["duration_seconds"]
        .as_f64()
        .filter(|d| d.is_finite() && *d >= 0.))
}
pub(crate) fn position(value: Option<&Value>, duration: Option<f64>) -> Result<Option<f64>> {
    value
        .map(|value| {
            value
                .as_f64()
                .filter(|v| v.is_finite() && *v >= 0. && *v <= duration.unwrap_or(604800.))
                .ok_or_else(|| bad("position outside item timeline"))
        })
        .transpose()
}
pub(crate) fn token(h: &HeaderMap) -> String {
    h.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';')
                .find_map(|s| s.trim().strip_prefix("oxevu_session="))
        })
        .unwrap_or_default()
        .to_owned()
}
pub(crate) fn touch(room: &mut Room, u: &Value, h: &HeaderMap) {
    let now = Instant::now();
    room.touched = now;
    room.members.insert(
        u["id"].as_str().unwrap_or_default().into(),
        Member {
            username: u["username"].as_str().unwrap_or_default().into(),
            token: token(h),
            touched: now,
        },
    );
}
// Room state and validation share one lock so membership and host controls are atomic.
pub(crate) async fn prune(a: &App, rooms: &mut BTreeMap<String, Room>) -> Result<()> {
    let mut closed = Vec::new();
    for (id, room) in rooms.iter_mut() {
        if prune_room(a, room).await? {
            closed.push(id.clone());
        }
    }
    for id in closed {
        rooms.remove(&id);
    }
    Ok(())
}
pub(crate) async fn prune_one(a: &App, rooms: &mut BTreeMap<String, Room>, id: &str) -> Result<()> {
    if let Some(room) = rooms.get_mut(id)
        && prune_room(a, room).await?
    {
        rooms.remove(id);
    }
    Ok(())
}
async fn prune_room(a: &App, room: &mut Room) -> Result<bool> {
    if room.touched.elapsed() > ROOM_IDLE {
        return Ok(true);
    }
    let mut removed = Vec::new();
    for (user, m) in &mut room.members {
        let valid_session:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sessions WHERE id=? AND user_id=? AND expires>unixepoch())").bind(&m.token).bind(user).fetch_one(&a.pool).await.map_err(db)?;
        let profile = users::profile(a, user).await?;
        if !valid_session
            || m.touched.elapsed() > MEMBER_IDLE
            || access(a, profile.clone(), &room.item).await.is_err()
        {
            removed.push(user.clone());
        } else {
            m.username = profile["username"].as_str().unwrap_or_default().into();
        }
    }
    for user in &removed {
        room.members.remove(user);
    }
    if !removed.is_empty() {
        room.revision += 1;
    }
    Ok(!room.members.contains_key(&room.host))
}
