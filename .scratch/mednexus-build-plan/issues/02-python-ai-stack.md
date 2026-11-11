Type: research
Status: open
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
