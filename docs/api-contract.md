# Rust ↔ Python API contract

The contract between the **Rust API service** and the **Python AI service**. Decided in
[Rust↔Python boundary and API contract](https://github.com/PetarBrajkovic/med-nexus/issues/12);
the reasoning is in [ADR 0002](./adr/0002-rust-orchestrates-python-stateless.md).

This file describes **what goes over the wire**: endpoints, purpose, and example JSON.
It deliberately contains no Rust or Python types or signatures; designing those is
part of the work. Once the Python side exists, FastAPI's auto-generated `/docs` page
is a check that it matches this file. If they disagree, fix one of them; don't let
them drift.

## Ground rules

- **Who talks to whom.** Browser → Rust only. Rust → Python only. Python is never
  called by the browser, so it needs no CORS and no auth.
- **Rust serves the frontend.** The static files in `frontend/` are served by Rust
  on the same host and port as `/api`. Same origin means no CORS anywhere.
- **Python binds `127.0.0.1`.** No auth between services (single machine). If it were
  ever deployed on a network, a shared-secret header would be the next step.
- **Python is stateless.** It never touches MySQL and keeps no jobs or sessions.
  Its only persistent state is Qdrant (vectors + chunk text).
- **MySQL is Rust's alone.** Anything Python produces that must be kept (document
  metadata, summaries) comes back in a response, and Rust saves it.
- **Documents are identified by a UUID that Rust generates.** Rust saves the document
  row (`status = pending`) **before** asking Python to index it, so Qdrant never
  holds vectors that no MySQL row knows about.
- **JSON** everywhere except `/pdf/extract` (multipart). Field names are `snake_case`.

## Timeouts (set on Rust's HTTP client, per call)

| Call | Timeout | Why |
|---|---|---|
| `/answer` | ~180 s | one CPU LLM call, typically 30–60 s |
| `/summarize` | ~15 min | map-reduce over one long PDF, ~16 LLM calls |
| everything else | ~60 s | bounded work (one batch) |

## Errors

Python uses FastAPI's default error body with a meaningful HTTP status:

```json
{ "detail": "PDF contains no extractable text" }
```

- `422`: bad input (malformed request, unreadable PDF, unknown PMID)
- `503`: a dependency is down (Ollama, Qdrant, PubMed)
- `500`: a bug

Rust **never forwards Python's error body to the browser**. It maps every failure
into its own error type and returns one browser-facing shape:

```json
{ "error": { "code": "ai_unavailable", "message": "The AI service is not responding." } }
```

| What happened on the Python call | Rust returns to the browser |
|---|---|
| timed out | `504` |
| connection refused | `502` |
| Python `4xx` | `422` |
| Python `5xx` | `502` |

## Ingest flow (Rust drives it)

An **ingest job** is a row in MySQL. Rust runs it in a background task, makes short calls to
Python, and records progress in MySQL. The browser polls Rust for status.

**PubMed import:**

1. `/pubmed/resolve`: query or PMIDs → final PMID list
2. in batches of ≤ 200: `/pubmed/fetch` → Rust saves document rows (UUID, `pending`) →
   `/index` → Rust marks them `indexed` (or `failed`)

**PDF upload:**

1. Rust saves the file as `data/uploads/{uuid}.pdf` and the document row (`pending`)
2. `/pdf/extract` (sends the bytes) → text + pages/sections
3. `/index` → `indexed`
4. `/summarize` → Rust saves section and document summaries

**Failures:** if a call fails, the affected documents are marked `failed` with the error
message, and the job **continues** with the next batch. It ends as "completed with N
failures". There are no automatic retries; a manual "retry failed" re-runs only the
failed documents. This is safe because `/index` is idempotent.

---

## Endpoints

### `GET /health`

Liveness, plus whether Qdrant and Ollama are reachable.

```json
{ "status": "ok", "qdrant": true, "ollama": true }
```

### `POST /pubmed/resolve`

Turn a search query or a PMID list into the final PMID list to import, optionally
expanded with PubMed "similar articles" (this is how the evaluation distractors are
pulled in). Exactly one of `query` / `pmids` is given.

```json
{ "query": "metformin gestational diabetes", "max_results": 100 }
```
```json
{ "pmids": ["21645374", "18802093"], "similar_per_pmid": 10 }
```
Response (deduplicated, inputs first):
```json
{ "pmids": ["21645374", "18802093", "19156208", "..."] }
```

### `POST /pubmed/fetch`

Fetch and parse up to 200 PMIDs. PMIDs that PubMed doesn't return are listed in `missing`.

```json
{ "pmids": ["21645374", "18802093"] }
```
```json
{
  "documents": [
    {
      "pmid": "21645374",
      "title": "Do mitochondria play a role in remodelling lace plant leaves during programmed cell death?",
      "abstract": "Programmed cell death (PCD) is ...",
      "journal": "BMC Plant Biology",
      "year": 2011,
      "authors": ["Lord CE", "Wertman JN", "..."],
      "mesh_terms": ["Mitochondria", "Apoptosis", "..."]
    }
  ],
  "missing": []
}
```

### `POST /pdf/extract`

`multipart/form-data` with one field, `file` (the PDF bytes). Returns the text split
by page; `sections` is best-effort and may be empty.

```json
{
  "page_count": 12,
  "pages": [ { "page": 1, "text": "..." }, { "page": 2, "text": "..." } ],
  "sections": [ { "title": "Introduction", "start_page": 1, "text": "..." } ]
}
```

### `POST /index`

Chunk, embed (dense + sparse) and upsert into Qdrant a batch of documents that
already have a MySQL row. **Idempotent:** a document's existing points are replaced,
so indexing the same `document_id` twice never duplicates chunks.

Each chunk's Qdrant payload holds `document_id`, `chunk_index`, the **verbatim chunk
text** (what the evidence panel quotes), and `page`/`section` when known. Titles,
PMIDs and so on are **not** copied in; Rust looks them up in MySQL.

```json
{
  "documents": [
    {
      "document_id": "6f1c2a9e-3b7d-4c55-9a51-2f0d8e7b1c44",
      "pages": [ { "page": null, "section": "Abstract", "text": "Programmed cell death (PCD) is ..." } ]
    }
  ]
}
```
Response, one entry per document. A failure for one document doesn't fail the batch:
```json
{
  "results": [
    { "document_id": "6f1c2a9e-...", "status": "indexed", "chunk_count": 3 },
    { "document_id": "0b7e...",      "status": "failed",  "error": "empty text" }
  ]
}
```

### `POST /summarize`

Hierarchical (map-reduce) summary of one long document. **PDF uploads only**; whether
abstracts skip it is still open on the map. Long-running; see Timeouts.

```json
{
  "document_id": "6f1c2a9e-...",
  "sections": [ { "title": "Introduction", "text": "..." }, { "title": "Methods", "text": "..." } ]
}
```
```json
{
  "section_summaries": [ { "title": "Introduction", "summary": "..." } ],
  "document_summary": "..."
}
```

### `DELETE /documents/{document_id}`

Remove all of a document's points from Qdrant. Rust calls this before deleting the
MySQL row. Returns `204`, including when nothing was there.

### `POST /search`

Hybrid retrieval (dense + sparse, fused), then rerank. No LLM. Used by `/answer`
internally and directly by the evaluation (Recall@k, MRR).

```json
{ "query": "Does metformin reduce macrosomia in gestational diabetes?", "top_k": 8 }
```
```json
{
  "chunks": [
    {
      "document_id": "6f1c2a9e-...",
      "chunk_index": 1,
      "text": "Neonatal macrosomia was less frequent in the metformin group ...",
      "page": null,
      "section": "Abstract",
      "score": 0.87
    }
  ]
}
```

### `POST /answer`

Search, check abstention, then one LLM call. Rust calls only this endpoint for users.
`use_retrieval: false` skips retrieval entirely; it exists for the no-RAG baseline in
the evaluation.

```json
{ "question": "Does metformin reduce macrosomia in gestational diabetes?", "top_k": 8, "use_retrieval": true }
```
Answer:
```json
{
  "abstained": false,
  "answer": "Evidence suggests metformin is associated with less macrosomia than insulin [1][2] ...",
  "key_findings": [
    { "text": "Lower rate of macrosomia vs insulin.", "citations": [1] }
  ],
  "citations": [
    { "marker": 1, "document_id": "6f1c2a9e-...", "chunk_index": 1 },
    { "marker": 2, "document_id": "0b7e...",      "chunk_index": 0 }
  ],
  "chunks": [ { "document_id": "6f1c2a9e-...", "chunk_index": 1, "text": "...", "page": null, "section": "Abstract", "score": 0.87 } ]
}
```
Abstention (best score below the threshold, so no LLM call is made):
```json
{
  "abstained": true,
  "reason": "insufficient_evidence",
  "answer": null,
  "key_findings": [],
  "citations": [],
  "chunks": [ "...the low-scoring chunks, for transparency..." ]
}
```

Rust adds the title, PMID and link to each citation from MySQL, then builds the evidence
panel. The abstention threshold lives in Python; Rust only reads `abstained`.

---

## Still open (tracked on the map, not here)

- Whether `/search` / `/answer` filter by owner for **private uploads**
- Exact chunk size and section-aware chunking
- How the **evaluation harness** calls `/search` and `/answer` (directly or through Rust)
- Streaming answers (SSE) as a later UX improvement
