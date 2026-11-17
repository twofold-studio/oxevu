use super::*;
use base64::Engine;

pub(crate) async fn save(
    State(a): State<App>,
    h: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>> {
    let account = user(&a, &h).await?;
    let id = account["id"].as_str().ok_or_else(users::forbidden)?;
    let image = body["image"]
        .as_str()
        .ok_or_else(|| bad("image is required"))?;
    let encoded = image
        .strip_prefix("data:image/png;base64,")
        .filter(|value| value.len() <= 350_000)
        .ok_or_else(|| bad("avatar must be a PNG under 256 KiB"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| bad("invalid avatar encoding"))?;
    if bytes.len() < 24
        || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
        || bytes[16..20] != 256_u32.to_be_bytes()
        || bytes[20..24] != 256_u32.to_be_bytes()
    {
        return Err(bad("avatar must be a 256 by 256 PNG"));
    }
    sqlx::query("INSERT INTO user_avatars(user_id,image) VALUES(?,?) ON CONFLICT(user_id) DO UPDATE SET image=excluded.image")
        .bind(id).bind(image).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"user": users::profile(&a, id).await?})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn avatar_is_private_persistent_and_validated() {
        let a = crate::tests::app().await;
        let owner = crate::tests::account(&a, "owner", false).await;
        let other = crate::tests::account(&a, "other", false).await;
        assert!(
            save(
                State(a.clone()),
                HeaderMap::new(),
                Json(json!({"image":"invalid"}))
            )
            .await
            .is_err()
        );
        assert!(
            save(
                State(a.clone()),
                owner.clone(),
                Json(json!({"image":"data:image/svg+xml;base64,AAAA"}))
            )
            .await
            .is_err()
        );
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        bytes.extend_from_slice(&256_u32.to_be_bytes());
        bytes.extend_from_slice(&256_u32.to_be_bytes());
        let image = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        );
        let result = save(
            State(a.clone()),
            owner.clone(),
            Json(json!({"image":image})),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(result["user"]["avatar"], image);
        assert_eq!(user(&a, &owner).await.unwrap()["avatar"], image);
        assert!(user(&a, &other).await.unwrap()["avatar"].is_null());
    }
}
