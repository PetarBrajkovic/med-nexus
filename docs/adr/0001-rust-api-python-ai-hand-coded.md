# Rust API service, Python AI service, both hand-coded

The original plan named Java 21 + Spring Boot for the API service. We replaced it with **Rust**, kept the **Python** AI service, and changed nothing else about the architecture: the Rust service owns auth, MySQL, uploads, history, audit and orchestration, and talks to the Python service over REST. The reason is that the author wants to learn Rust, and the thesis is also a learning vehicle. For the same reason, both services are written by hand without AI codegen, task by task, and each task is reviewed afterwards. The frontend is the exception: it is plain HTML/CSS/JS, kept deliberately thin, and agent-written because it exists only for demonstration.

The thesis scope (literature sources, RAG, two-phase summarization, output format, English only) is unaffected. Only the implementation language changed.

## Consequences

- Delivery is slower: the author is a beginner in both languages. The backlog is ordered MVP-first (an end-to-end slice before auth/history/audit) so that a future deadline can cut from the end without breaking the core.
- Docker Compose is deferred. Infrastructure (MySQL, Qdrant, Ollama) runs natively on Windows during development.
