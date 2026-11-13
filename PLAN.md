# MedNexus — Project Plan

The technical scope and design of MedNexus. Planning and progress are tracked on the
wayfinder map: [MedNexus build plan (issue #7)](https://github.com/PetarBrajkovic/med-nexus/issues/7).
Architecture decisions live in [`docs/adr/`](./docs/adr/).

The scope in Section 3 is fixed. If any of it changes materially during
implementation, record the change and the reason in the change log rather than
silently drifting.

### Change log

- **Backend language: Java/Spring Boot → Rust.** The architecture is unchanged:
  Rust takes the API service's role, and Python FastAPI keeps the AI work. Both
  services are hand-coded (no AI codegen) and reviewed task by task. See
  `docs/adr/0001-rust-api-python-ai-hand-coded.md`.
- **Frontend: React → plain HTML/CSS/JS → TypeScript (no framework).** Kept thin,
  for demonstration only. Compiled with plain `tsc` (no bundler) into
  `frontend/dist`, which Rust serves. Decided in
  [MVP slice and backlog order](https://github.com/PetarBrajkovic/med-nexus/issues/16).
- **Docker Compose deferred.** MySQL, Qdrant and Ollama run natively on Windows
  during development. Compose is an optional last step.

---

## 1. Project identity

**Title:** MedNexus: An AI-Powered Clinical Knowledge Assistant Based on
Retrieval-Augmented Generation (RAG)

**One-line description:** A locally-run, fully open-source platform that helps
healthcare professionals find, retrieve and understand evidence from medical
literature, using semantic search and RAG, with every claim traceable to a source.

**Core design philosophy — do not drift from this:**

> MedNexus is **not an AI doctor.** It does not diagnose. It retrieves evidence,
> summarizes trusted sources, cites them, and supports clinical judgment without
> replacing it.

This framing was a deliberate decision (see 3.1). Reject any feature request
that pulls toward autonomous diagnosis.

---

## 2. Positioning

This is a **software engineering thesis that integrates AI**, not a machine
learning thesis. No models are trained from scratch.

- ~70% software engineering — architecture, REST APIs, authentication, database
  design, service communication, document processing pipeline, testing,
  logging, documentation.
- ~30% AI — embeddings, semantic search, vector databases, RAG, local LLM
  inference, prompt design, evidence-grounded summarization.

---

## 3. Scope decisions

### 3.1 Scope: knowledge assistant, not diagnostic AI

Diagnostic AI cannot be validated in a student project and raises
ethics/safety questions. A retrieval-and-evidence assistant avoids this entirely.

### 3.2 Literature sources

- **PubMed / MEDLINE** — ~38M abstracts. Free via the E-utilities API or the
  annual baseline dump.
- **PubMed Central Open Access Subset** — full text. **Only the OA subset is
  licensed for bulk download and redistribution.** The rest of PMC is not — do
  not scrape it.
- **MeSH** terms as structured metadata for filtering.
- A small curated set of clinical guidelines (WHO, NICE) — check each one's terms
  individually.
- User-uploaded PDFs into a private knowledge base.

Abstracts come from **PubMed**; full text comes from **PMC OA**. These are
different databases. Do not conflate them.

### 3.3 Retrieval approach

Hybrid retrieval, then rerank:

1. Embed the query.
2. **Dense** (vector/semantic) search in Qdrant **plus lexical BM25 search** in
   parallel. Lexical matters because medical text is full of exact tokens — drug
   names, dosages, ICD codes — where embeddings underperform.
3. Merge, then **rerank** the top ~20 with a cross-encoder (`bge-reranker-base`)
   down to the top 5–8.
4. Those 5–8 chunks are the only context the LLM ever sees.

### 3.4 Abstention

If the best retrieval scores fall below a threshold, the system states that
there is insufficient evidence in the knowledge base rather than generating an
answer. This is a safety feature.

### 3.5 Citation integrity

The evidence panel quotes **original chunk text**, never a generated summary.
Summaries are lossy — they drop dosages, numbers and caveats. Citing a summary
means citing something the model wrote, which defeats the purpose of RAG.

### 3.6 Databases — polyglot persistence

- **MySQL** — users, roles, documents, metadata, precomputed summaries, search
  history, audit logs.
- **Qdrant** — chunk embeddings and vector search metadata.

### 3.7 Language scope

**English only.** Serbian is a possible extension, not a promise. See Section 7.

### 3.8 Cost and hosting

Target €0. Everything runs locally. No paid APIs, no proprietary services, no
data leaving the machine.

### 3.9 No patient data

Public literature and user-uploaded documents only. No PHI, no personal data
processing, therefore no ethics approval required.

---

## 4. Architecture

```
                  HTML / CSS / JS
                           |
                           v
                       Rust API
                           |
             +-------------+-------------+
             |                           |
             v                           v
           MySQL                  Python FastAPI
                                         |
                              +----------+----------+
                              |                     |
                              v                     v
                       Embedding Model          Local LLM
                              |
                              v
                           Qdrant
```

The Rust API owns business logic, auth, and relational data. Python owns
everything AI (embeddings, retrieval, reranking, LLM inference) because the
tooling is Python-first. They talk over REST.

### 4.1 Technology stack

| Layer | Choice |
|---|---|
| Frontend | TypeScript + HTML/CSS (no framework; plain `tsc`, no bundler) |
| Backend | **Rust** (crate stack: see the wayfinder map) |
| Relational DB | MySQL |
| AI service | Python + FastAPI |
| Embeddings | `BAAI/bge-small-en-v1.5` |
| Reranker | `bge-reranker-base` (cross-encoder) |
| Lexical search | BM25 |
| Vector DB | Qdrant |
| LLM runtime | Ollama (llama.cpp underneath) |
| LLM | Llama 3.2 3B / Gemma 3 4B / Qwen 3B — quantized |
| Deployment | Native on Windows; Docker Compose optional, at the end |

### 4.2 Target hardware

Ryzen 5 7600X, 32 GB RAM, **integrated GPU only — no dedicated GPU.**
Every performance decision in this document follows from that constraint.

**Realistic expectations:** roughly 8–15 tokens/sec generation on a quantized
3–4B model. Prompt processing over a 3k-token context is the real bottleneck.
Expect roughly 30–60 seconds per answer. Design around this; do not promise
interactivity.

---

## 5. Summarization design — the two-phase split

There are two different problems that both get called "summarization", and only
one of them is actually summarization.

### 5.1 "Summarize the whole corpus" — never done

A clinician never asks to summarize all of PubMed; they ask a specific question.
Corpus scale is handled by **retrieval, not summarization** — find the 5–8
passages that matter, ignore the rest. This is why **response time does not grow
with corpus size.** 1,000 documents or 1,000,000, the model reads ~3,000 tokens.

### 5.2 "This one document doesn't fit" — hierarchical (map-reduce)

A 40-page guideline is ~20,000 words, roughly 26,000 tokens. The practical CPU
context budget is ~4,000 tokens. So:

```
Document (26k tokens)
   |
   +-- split into sections
   |
   +-- MAP:    summarize each section  -> ~200 tokens each  (~15 LLM calls)
   |
   +-- REDUCE: feed the section summaries (~3k tokens) into one final call
                  |
                  +-- document summary
```

Repeat the reduce step if needed. Two levels is enough in practice.

### 5.3 When each phase runs

**At ingest (background, once per document, nobody is waiting):**

1. Extract text from PDF.
2. Chunk into ~400-token passages with ~50-token overlap.
3. Compute an embedding per chunk, store in Qdrant.
4. Run map-reduce summarization, store section + document summaries in MySQL.

**At query time (must be fast):**

1. Embed the question.
2. Hybrid retrieve top ~20, rerank to top 5–8.
3. **One** LLM call over those chunks, producing answer + citations.

The ingest example above costs ~16 LLM calls, i.e. several minutes on CPU. That
is acceptable because it happens once, offline. Query time costs **one** call.

### 5.4 Why not a long-context (128k) model?

1. **Speed** — prompt processing scales with context length; 128k on CPU is
   minutes per question.
2. **Accuracy** — long-context models degrade ("lost in the middle"); facts
   buried mid-context get missed. 5 good passages beat 200 mediocre ones.
3. **Citations** — with 5 retrieved chunks you know exactly which document each
   claim came from. With a 128k dump you do not.

---

## 6. Evaluation plan — the thesis contribution

This is what turns "I built an app" into a thesis, and it shows the system is
more than a wrapper around an existing LLM.

1. **Retrieval quality** — Recall@k and MRR on a labeled question-to-document set.
2. **Baseline comparison** — the same LLM answering **with** retrieval vs.
   **without** retrieval. If RAG wins measurably, that is shown with data rather
   than assertion.
3. **Citation faithfulness** — what fraction of claims in an answer are actually
   supported by a retrieved passage.
4. **Ablations** — dense-only vs. hybrid; with vs. without reranker.

**Benchmarks:** use existing labeled sets to avoid hand-labeling everything —
**PubMedQA**, **BioASQ**, and the **MIRAGE** benchmark from the MedRAG toolkit.

---

## 7. Language scope

**In scope: English only.**
Rationale: (a) the medical literature is almost entirely English; (b) small
open-source models that run on CPU are markedly more reliable in English,
especially for technical terminology.

**Out of scope, documented as a possible extension:** Serbian.

**Known trap — do not implement Serbian naively.** `bge-small-en-v1.5` is
**English-only**. A Serbian query embedded with it produces a meaningless vector
and retrieval fails *silently*. Two valid approaches if Serbian is ever added:

1. **Translate-then-retrieve** — Serbian query, translated to English by the local
   LLM, then retrieve, then generate the answer in Serbian. Show the translated
   query in the UI for transparency. Simpler.
2. **Multilingual embeddings** — `BAAI/bge-m3` or `multilingual-e5-base`, which
   do genuine cross-lingual retrieval. Heavier on CPU.

Either way, evaluate it separately rather than assuming it works.

---

## 8. Output format

A response to a clinician's query contains:

1. A short direct answer with inline citation markers `[1][2]`.
2. Key findings as bullets, each traceable to a source.
3. An **evidence panel**: verbatim passage, document title, section/page, PMID
   plus link, relevance score.
4. **Explicit abstention** when retrieval confidence is below threshold.
5. A persistent disclaimer: decision *support*, not diagnosis.

---

## 9. Feature list

**Core (in scope):**

- Authentication and user roles
- PDF upload
- Text extraction, chunking, embedding generation
- Hybrid semantic + lexical search, with reranking
- RAG question answering with citations
- Document summarization (ingest-time, hierarchical)
- Evidence panel
- Search history
- Audit logging

**Explicitly out of scope** — deliberately cut to keep the scope defensible.
Do not re-add without a reason: X-ray analysis, ECG interpretation, drug
interaction checking, OCR, speech-to-text, knowledge graphs, medical imaging.

---

## 10. Open questions

- [ ] Corpus size for the demo — the full PubMed baseline is large. A focused
      subset (e.g. one specialty) is probably the right call. Decide before
      ingestion.
- [ ] Exactly which clinical guidelines, and confirm each one's license.
- [ ] Chunking strategy: fixed-size vs. section-aware. Section-aware is better for
      structured guidelines but needs more parsing work.
