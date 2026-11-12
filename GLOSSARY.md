# MedNexus

A local evidence assistant that retrieves, cites and summarizes medical literature to support (never replace) clinical judgment.

## People and accountability

**User**:
A person with an account who asks questions, uploads PDFs, and owns their own search history.
_Avoid_: Account (for the person), member, client

**Admin**:
A user who also curates the knowledge base (PubMed import, document deletion), manages other users' roles, and reads the audit log.
_Avoid_: Superuser, moderator, curator

**Search history**:
A user's own record of the questions they asked, each kept together with the answer and evidence exactly as they were shown at the time; visible only to that user.
_Avoid_: Query log, activity log

**Audit log**:
An append-only record of security- and knowledge-base-relevant events (who did what, when), kept for accountability; it records that a question was asked but not its text.
_Avoid_: Search history, log (alone), event log

## Knowledge and ingest

**Knowledge base**:
The full set of documents the assistant can retrieve from and cite.
_Avoid_: Corpus (except when talking about the evaluation set), database, index

**Ingest path**:
A route by which a document enters the knowledge base: PubMed import or PDF upload.
_Avoid_: Import (alone), source, pipeline

**PubMed import**:
The ingest path that pulls abstracts and their metadata from PubMed by search query or PMID.
_Avoid_: PubMed fetch, crawl, scrape

**PDF upload**:
The ingest path where a user adds their own PDF document.
_Avoid_: File import

**Ingest job**:
One run of an ingest path over one or more documents, tracked from start to finish; some of its documents may succeed while others fail. A job cut short by a server restart is **interrupted**, and can be retried.
_Avoid_: Import job, task, pipeline run

**Chunk**:
A short passage of a document's original text, the unit that is retrieved and quoted verbatim as evidence.
_Avoid_: Passage (in code), segment, snippet

## Answering

**Abstention**:
The assistant's explicit statement that the knowledge base holds insufficient evidence, given instead of an answer.
_Avoid_: Refusal, no-answer, fallback

## Evaluation

**Gold document**:
The document a benchmark question is known to be answered by; retrieval succeeds only if it is found.
_Avoid_: Ground truth document, target, answer document

**Distractor**:
A document in the knowledge base that is not the gold document for a given question, added on purpose so retrieval must pick the needle out of a realistic haystack.
_Avoid_: Noise document, negative

**Tune split**:
The half of the benchmark questions used to set the system's adjustable parameters; its numbers are never reported as results.
_Avoid_: Dev set, train set, validation set

**Report split**:
The other half of the benchmark questions, run only with already-tuned settings; every reported result comes from it.
_Avoid_: Test set, holdout (ambiguous with held-out question)

**Held-out question**:
A benchmark question whose gold document is deliberately left out of the knowledge base, so the correct behaviour is abstention.
_Avoid_: Unanswerable question, negative question
