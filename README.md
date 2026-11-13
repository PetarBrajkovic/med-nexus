# MedNexus

A locally-run clinical knowledge assistant built on Retrieval-Augmented Generation (RAG).
It retrieves evidence from medical literature, cites it, and supports clinical judgment
without replacing it. **Not an AI doctor.**

See [PLAN.md](./PLAN.md) for scope and design. Architecture decisions are in [docs/adr/](./docs/adr/).

## Layout

| Folder | What | Language | Written by |
|---|---|---|---|
| `backend/` | API service: auth, MySQL, uploads, history, audit, orchestration | Rust | hand-coded |
| `ai-service/` | AI service: ingest, chunking, embeddings, hybrid retrieval, reranking, LLM, evaluation | Python | hand-coded |
| `frontend/` | Thin demo UI, TypeScript (plain `tsc`), no framework | TypeScript | agent |
| `docs/` | ADRs and research notes | — | — |

## Local infrastructure (native, no Docker)

- MySQL 8.4
- Qdrant (Windows binary)
- Ollama (Windows)

Setup instructions will be added as the toolchain gets installed.
