-- Fill missing imported catalog metadata without overwriting owner matches.
-- Artwork stays on the authenticated same-origin acquisition proxy.
WITH imported AS (
    SELECT i.item_id, t.data AS title, t.provider_id, t.kind
    FROM acquisition_imports i
    JOIN acquisition_titles t ON t.id = i.title_id
    JOIN item_metadata m ON m.item_id = i.item_id
    WHERE i.state = 'scanned'
      AND json_extract(m.data, '$.provider_ids.tmdb') = t.provider_id
      AND json_extract(t.data, '$.poster_url') LIKE '/api/v1/acquisition/artwork/%.jpg'
)
UPDATE item_metadata
SET data = json_patch(
    (SELECT json_object(
        'poster_url', json_extract(title, '$.poster_url'),
        'genres', json((SELECT json_group_array(json_extract(value, '$.name'))
                       FROM json_each(title, '$.genres')
                       WHERE json_extract(value, '$.name') IS NOT NULL)),
        'external_urls', json_object('tmdb', 'https://www.themoviedb.org/' || kind || '/' || provider_id)
    ) FROM imported WHERE imported.item_id = item_metadata.item_id LIMIT 1),
    data
)
WHERE item_id IN (SELECT item_id FROM imported);
