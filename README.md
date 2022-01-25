# `do-sqlite`

[Experimental] POC that persists SQLite in a Cloudflare Durable Object.

My journey of creating it: https://ma.rkusa.st/store-sqlite-in-cloudflare-durable-objects

## Next Steps

- [ ] Add support for persistent journal files (to prevent data loss and database corruption)
- [ ] Add proper error handling
- [ ] Ensure that everything works fine when the DO is used concurrently

## Usage

Deploy to Cloudflare (you might have to get a script limit increase to be able to deploy it):

```bash
wrangler publish
```

Execute a query (path pattern: `/:database/{query,execute}`):

```bash
curl -i -X POST -H 'Content-Type: application/json' \
  -d '{"sql":"CREATE TABLE vals (id INTEGER PRIMARY KEY AUTOINCREMENT, val VARCHAR NOT NULL)"}' \
  https://do-sqlite.YOUR_WORKERS.workers.dev/main/execute
```

```bash
curl -i -X POST -H 'Content-Type: application/json' \
  -d '{"sql":"INSERT INTO vals (val) VALUES (?1)","params":["val"]}' \
  https://do-sqlite.YOUR_WORKERS.workers.dev/main/execute
```

```bash
curl -i -X POST -H 'Content-Type: application/json' \
  -d '{"sql":"SELECT * FROM vals"}' \
  https://do-sqlite.YOUR_WORKERS.workers.dev/main/query
```
