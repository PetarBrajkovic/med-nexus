Type: research
Status: resolved
Blocked by: —

# Python AI stack and Python 3.14 compatibility

## Question

Which libraries should the Python AI service use, and does the installed **Python 3.14.7** work with them on Windows (CPU only)? Check against official sources (PyPI wheels, release notes):

- FastAPI + uvicorn
- sentence-transformers + torch (CPU wheels for 3.14?) for `BAAI/bge-small-en-v1.5`; alternative: `fastembed` (ONNX, no torch)
- Reranker `bge-reranker-base` (cross-encoder): via sentence-transformers or fastembed
- BM25 / hybrid: Qdrant's native sparse vectors + server-side fusion (Query API, RRF) vs separate `rank-bm25`/`bm25s` in Python
- `qdrant-client` and the Qdrant Windows binary
- PDF extraction: PyMuPDF vs pypdf vs pdfplumber (license! PyMuPDF is AGPL)
- Ollama Python client vs plain HTTP
- PubMed E-utilities: Biopython Entrez vs plain HTTP
- Project/venv tooling for a beginner: venv+pip vs uv

Output: a recommended stack, a verdict on Python 3.14 (stay, or install 3.12/3.13 alongside), and links to docs.

## Answer

Recommendation (details and sources: `docs/research/python-ai-stack.md` on the `research/python-ai-stack` branch, never merged, read with `git show research/python-ai-stack:docs/research/python-ai-stack.md`):

- **Python 3.14.7 stays.** All dependencies have Windows cp314 wheels or are pure Python (resolver check, including torch 2.15 cpu). A real install/import is still to be confirmed. Fallback: Python 3.13 is already installed (`py -3.13 -m venv`).
- **fastembed 0.9** (ONNX, no torch): bge-small-en-v1.5, bge-reranker-base, BM25 sparse. Fallback: sentence-transformers + CPU torch
- **Qdrant 1.20** (native Windows zip): dense + sparse vectors, Query API prefetch + **server-side RRF fusion**, so no separate BM25 index in Python
- **FastAPI + uvicorn**; **pypdf** (BSD) for PDFs (PyMuPDF is AGPL, so avoid it); the **ollama** client; **httpx** for PubMed E-utilities (3 req/s without a key, 10/s with one)
- Tooling: plain **venv + pip**
- Open: pypdf quality on scientific PDFs; whether fastembed BM25 output matches Qdrant's sparse format (to verify with a prototype in the backlog)
