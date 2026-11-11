# Rust orchestrates; the Python AI service is stateless

Ingest takes minutes (11k abstracts to embed, plus map-reduce summarization of long PDFs), so some service has to run long jobs and track their progress. We put that in **Rust**: an ingest job is a MySQL row, Rust runs it in a background task, and it calls Python with short, bounded requests (fetch a batch, index a batch, summarize one document). **Python never touches MySQL and keeps no jobs.** Its only persistent state is Qdrant. Everything else it produces (document metadata, summaries) goes back to Rust in the response, and Rust saves it. Rust generates each document's UUID and saves the row *before* indexing, so Qdrant can never hold orphan vectors.

## Considered options

- **Python runs the job, Rust polls it.** Rejected: Python would need job state that a restart loses, and two services would own "what is ingested".
- **Python runs the job and calls back into Rust.** Rejected: a two-way dependency, so Python would have to know Rust's URL and auth.
- **Python writes its own MySQL tables.** Rejected: two services sharing one schema.

## Consequences

- AI logic stays in Python even when Rust drives the flow: abstention thresholds and map-reduce live behind `/answer` and `/summarize`. Rust does not chain search → generate itself.
- `/index` must be idempotent (replace a document's points) so that "retry failed" is safe.
- The contract is in `docs/api-contract.md`.
