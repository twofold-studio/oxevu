# Media removal and retained library entries

Admin-only `POST /api/v1/library/removals/preview` accepts
`{item_ids:string[],keep_metadata:boolean}` (1–1000 distinct IDs). `keep_metadata`
defaults true. Movies, seasons and series use the same endpoint: select the
corresponding episode IDs. No filesystem paths are accepted.

Response: `{token,count,keep_metadata,bytes_selected,shared_files_count,items,notice}`.
Each item is `{id,title,size_bytes:number|null,local_file:boolean,shared_links:number}`.
The token captures the selected items, options and source file identities. Preview
before enabling confirmation and show the count, size, shared-link warning and
exact titles. This is permanent deletion of the selected library media files only;
artwork, subtitle files and torrent download sources are retained.

Admin-only `POST /api/v1/library/removals` accepts the same fields plus
`{token:string,confirm:true}`. A stale preview, symlink, missing untracked file,
active player, active import/scan or invalid item refuses the operation. The
explicit confirm flag and current token are required. Never send on selection,
opening the dialog, or toggling keep metadata. Require an acknowledgement checkbox
and final destructive button. Forgetting metadata deletes all users' progress,
favorites and reactions for those entries; say this explicitly.

Result: `{removed_ids:string[],kept_metadata:boolean,failures:{id,message}[],
bytes_freed_estimate:number,shared_files_count:number}`. Display partial failures
honestly and refresh the catalog even after a partial result. A shared hard link
can mean no space is reclaimed; don't label selected bytes as freed space.

Catalog Items add optional `file_available:boolean` (missing means true),
`file_removed_at:number|null` (Unix seconds), and `reaction:'like'|'dislike'|null`.
Kept entries stay in normal movie/series/season browsing with their artwork,
metadata and watch history, labelled **No local file**. Disable Play, Resume,
party creation and watch links for missing files. Exclude them from Continue
watching/Next up play targets while keeping metadata details and watched status.

Per-user `PUT /api/v1/items/{id}/reaction` accepts `{value:'like'|'dislike'|null}`,
returns `{value}`. Use separate thumb controls labelled Your opinion; don't mix
personal opinion with the provider's numeric rating. Clearing a selected thumb
sends null. Only send when clicked. Existing favorite and watched state remain.

**Find again:** if an item has a positive numeric TMDB provider ID, navigate to
`href.discoverTitle(item.kind==='episode'?'tv':'movie', providerId)`, where the
existing release search/indexer controls and explicit Download action apply.
Otherwise navigate to `href.discover(title,kind)` for identification. For an
episode/season, explain which season/episode needs downloading rather than
claiming the search is already scoped. Expose links only to users who can manage
downloads. Never start a download on page entry or automatically from removal.

Intentionally removed units are excluded from automatic Wanted reacquisition.
Old completed torrents do not re-import deleted files automatically. An explicit
new manual download may reuse a retained seeding source, and a successful import
clears suppression for its imported units. A uniquely matching kept item in the
same library retains its ID, opinions and watch history after import, including
when the new release has a different filename or extension.

No live media is to be deleted during development. Use only generated fixtures.
Targeted checks and builds suffice; the owner explicitly skips live UI/playback
checks and full-suite validation by default.
