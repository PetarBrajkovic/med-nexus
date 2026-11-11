Label: wayfinder:map

# MedNexus build plan

## Destination

A locked architecture for MedNexus (Rust API service + Python AI service + plain HTML/CSS/JS frontend) **and an ordered backlog of small, hand-codeable tasks** for both services, written to `.scratch/backlog/`. Once the backlog exists the map is done, and building proceeds task by task with a review after each one.

## Notes

- **Domain:** a thesis project (software engineering + RAG). `PLAN.md` holds the approved scope; `docs/adr/0001-rust-api-python-ai-hand-coded.md` explains the language switch. Do not change the approved scope (PLAN.md §4).
- **The human is a beginner in Rust and Python** and codes both services **by hand without AI**. The agent's role is reviewer/teacher, and it writes no backend or AI-service code. The frontend is the agent's job.
- **Backlog task format:** goal · the one new concept being learned · acceptance criteria · links to official docs. **No code, no signatures.** Small, a single concept per task, especially at the start of Rust.
- **Backlog order: MVP-first.** End-to-end slice (upload → ingest → retrieval → answer with citations) first; auth/history/audit after.
- **Infrastructure runs natively on Windows** (MySQL 8.4 already installed; Qdrant .exe, Ollama). No Docker until the end.
- **Repo skeleton** (git, folders, README, .gitignore) was created by the agent. `cargo new` / venv setup are the human's first tasks.
- **Skills:** for grilling tickets, `grilling` + `domain-modeling`; when teaching or reviewing, `teaching-philosophy`.
- Machine: Ryzen 5 7600X, 32 GB RAM, iGPU only. Python 3.14.7 installed (compatibility is an open question).

## Decisions so far

<!-- one line per closed ticket -->

- [Rust crate stack for a beginner](issues/01-rust-crate-stack.md): axum + tokio, sqlx (MySQL), argon2, reqwest, tracing, thiserror; auth leans toward tower-sessions (for 07)
- [Python AI stack and Python 3.14 compatibility](issues/02-python-ai-stack.md): stay on 3.14; fastembed (ONNX), Qdrant sparse+dense with RRF fusion, FastAPI, pypdf, venv+pip

## Not yet specified

- **Chunking strategy:** fixed-size vs section-aware (PLAN.md §12). Depends on the corpus and the PDF extraction choice; the MVP probably uses fixed-size, with section-aware as a possible later step.
- **Ingest job model:** ingest takes minutes (map-reduce summarization on CPU). How is the background job run and tracked, and where does its status live? Depends on the Rust↔Python boundary.
- **Evaluation harness:** Recall@k / MRR, RAG vs no-RAG, citation faithfulness, ablations (PLAN.md §7). Which benchmark subset fits the chosen corpus, and how does the harness run? Depends on the corpus and the AI stack.
- **Frontend serving:** does Rust serve the static files, or is there a separate static server? Is CORS needed? Small; resolved together with the API contract.
- **Clinical guidelines:** which ones (WHO/NICE) and their licenses (PLAN.md §12). Probably not part of the MVP.
- **Docker Compose at the end:** whether it's needed for the defense/demo at all.

## Out of scope

<!-- work ruled beyond the destination -->
