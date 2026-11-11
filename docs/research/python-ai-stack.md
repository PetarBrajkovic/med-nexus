# Python AI service: library stack and Python 3.14 verdict

Status: research note, 2026-11. Each claim cites a primary source (PyPI JSON, package source, official docs, release page). "Dry run" = a local `pip install --dry-run` run for this note; nothing was installed into the project.

## Summary

- **Verdict: stay on Python 3.14.7.** Every package in the proposed stack has a cp314 Windows wheel or a pure-Python wheel, and a binary-only resolve for `win_amd64` / Python 3.14 succeeded for the full set (dry run). Python 3.13 is already installed at `C:\Users\Pex\AppData\Local\Programs\Python\Python313\python.exe` and is the fallback.
- **Embeddings, reranker, BM25 sparse vectors: `fastembed`** (ONNX, no torch). It ships `BAAI/bge-small-en-v1.5`, `BAAI/bge-reranker-base`, and a BM25 sparse class. `sentence-transformers` + CPU torch is the fallback.
- **Hybrid search: Qdrant server (v1.20.0, native Windows zip) with sparse vectors + Query API prefetch, fused with RRF server-side.**
- **Web/PDF/LLM/PubMed:** FastAPI + uvicorn, `pypdf` (BSD), `ollama` client, plain `httpx` for E-utilities.
- **Tooling for a beginner:** plain `venv` + `pip` with `py -3.14`.

## VERDICT: Python 3.14

**Stay on 3.14.7.** Reasons: all packages above have 3.14 Windows artifacts (see Findings 2–5), and PEP 745 keeps 3.14 on binary bugfix releases until 2027. Keep 3.13 as the fallback, already installed:

```
py -3.14 -m venv .venv          # project venv, default
.venv\Scripts\activate
py -3.13 -m venv .venv313       # fallback only, if a wheel is missing on 3.14
```

If a package fails on 3.14 (no wheel, import error), switch the venv to 3.13 rather than building from source. Build tools are not needed on the path above.

## Recommended stack

| Layer | Library / tool | Version (PyPI / release) | Why | Doc / source |
|---|---|---|---|---|
| Runtime | CPython | 3.14.7 (installed) | Matches the dev machine; binary installers supported until 2027 | [PEP 745](https://peps.python.org/pep-0745/) |
| Web API | fastapi | 0.162.4 | Pure-Python wheel, `requires_python >=3.10` | [PyPI JSON](https://pypi.org/pypi/fastapi/json) |
| ASGI server | uvicorn | 0.54.0 | Pure-Python wheel | [PyPI JSON](https://pypi.org/pypi/uvicorn/json) |
| Embeddings, reranker, BM25 | fastembed | 0.9.1 | ONNX, no torch; ships the three models needed | [PyPI JSON](https://pypi.org/pypi/fastembed/json), [repo](https://github.com/qdrant/fastembed) |
| ONNX runtime (via fastembed) | onnxruntime | 1.31.1 (cp314 win_amd64) | Required by fastembed on 3.14 (`>=1.24.2`) | [PyPI JSON](https://pypi.org/pypi/onnxruntime/json) |
| Fallback ML stack | sentence-transformers + torch CPU | 6.3.0 / 2.15.0+cpu | Use only if fastembed fails | [ST PyPI](https://pypi.org/pypi/sentence-transformers/json), [PyTorch CPU index](https://download.pytorch.org/whl/cpu/torch/) |
| Vector store | Qdrant server | v1.20.0 (`qdrant-x86_64-pc-windows-msvc.zip`) | Native Windows binary, no Docker | [GitHub release](https://github.com/qdrant/qdrant/releases/latest) |
| Vector client | qdrant-client | 1.20.0 (Apache-2.0) | Official client; local mode for tests | [PyPI JSON](https://pypi.org/pypi/qdrant-client/json), [README](https://github.com/qdrant/qdrant-client) |
| Lexical search | Qdrant sparse vectors (BM25 via fastembed `Bm25`) + Query API RRF | see above | Hybrid fusion runs in Qdrant, not in Python | [Hybrid queries](https://qdrant.tech/documentation/search/hybrid-queries/) |
| PDF text | pypdf | 7.0.1 (BSD-style) | Pure Python, permissive license | [PyPI JSON](https://pypi.org/pypi/pypdf/json), [LICENSE](https://github.com/py-pdf/pypdf/blob/main/LICENSE) |
| PDF text (alt) | PyMuPDF | 2.1 (AGPL-3.0 or commercial) | Often better layout quality; license problem (see below) | [PyPI JSON](https://pypi.org/pypi/pymupdf/json) |
| LLM client | ollama | 0.6.3 | Official client, typed, async option | [ollama-python README](https://github.com/ollama/ollama-python) |
| HTTP (PubMed) | httpx | 0.28.1 (BSD-3) | Already a dependency of ollama and fastapi tooling; plain E-utilities calls | [PyPI JSON](https://pypi.org/pypi/httpx/json) |
| Lexical (alt, in-Python) | bm25s | 0.3.16 (MIT) | Only if not using Qdrant sparse | [PyPI JSON](https://pypi.org/pypi/bm25s/json) |
| Env | venv + pip (`py -3.14 -m venv .venv`) | stdlib | Simplest for a beginner; no extra tool | [venv docs](https://docs.python.org/3/library/venv.html) |

## Findings

1. **Python 3.14 support horizon.** PEP 745: "Python 3.14 will receive bugfix updates approximately every two months for approximately 24 months." 3.14.7 is listed as released 2026-08-05; the last binary-installer bugfix is "3.14.16: Tuesday, 2027-10-05 (Final regular bugfix release with binary installers)". > quoted above, [PEP 745](https://peps.python.org/pep-0745/), "Release schedule".

2. **Torch cp314 Windows wheels exist.** PyPI `torch` 2.15.0 has `torch-2.15.0-cp314-cp314-win_amd64.whl` (and a free-threaded `cp314t` variant). The PyTorch CPU index also lists `torch-2.15.0+cpu-cp314-cp314-win_amd64.whl`. PyTorch's install page states support for "Python 3.10 - 3.14." > "Python 3.10 - 3.14." [PyTorch get-started](https://pytorch.org/get-started/locally/), Python version list. The PyPI Windows wheel's CUDA dependencies are marked `platform_system == "Linux"` only, so the PyPI Windows build carries no CUDA stack ([torch PyPI JSON](https://pypi.org/pypi/torch/json), requires_dist). Install from the CPU index to be explicit.

3. **sentence-transformers works on 3.14 but pulls torch.** `sentence-transformers` 6.3.0 is pure Python; it requires `torch>=2.2` and `transformers<6.0.0,>=5.0.0` ([PyPI JSON](https://pypi.org/pypi/sentence-transformers/json), requires_dist). `transformers` 5.22.2 is pure Python, and `scipy` 1.18.1, `scikit-learn` 1.9.1, `numpy` 2.5.4 and `tokenizers` 0.23.3 all have cp314 win_amd64 wheels ([transformers](https://pypi.org/pypi/transformers/json), [scipy](https://pypi.org/pypi/scipy/json), [scikit-learn](https://pypi.org/pypi/scikit-learn/json), [numpy](https://pypi.org/pypi/numpy/json), [tokenizers](https://pypi.org/pypi/tokenizers/json)).

4. **fastembed is torch-free and covers all three models.** fastembed's default text model is `model_name: str = "BAAI/bge-small-en-v1.5"` ([fastembed `text_embedding.py`](https://raw.githubusercontent.com/qdrant/fastembed/main/fastembed/text/text_embedding.py)). Its ONNX cross-encoder uses `model="BAAI/bge-reranker-base"` ([fastembed `onnx_text_cross_encoder.py`](https://raw.githubusercontent.com/qdrant/fastembed/main/fastembed/rerank/cross_encoder/onnx_text_cross_encoder.py)). It has a BM25 sparse class: `class Bm25(SparseTextEmbeddingBase):` ([fastembed `sparse/bm25.py`](https://raw.githubusercontent.com/qdrant/fastembed/main/fastembed/sparse/bm25.py)). On Python 3.14 it requires `onnxruntime>=1.24.2` and `numpy>=2.3.0` ([PyPI JSON](https://pypi.org/pypi/fastembed/json), requires_dist). Both fastembed (`py3-none-any`) and onnxruntime 1.31.1 (`cp314-cp314-win_amd64`) have 3.14 Windows artifacts ([onnxruntime PyPI JSON](https://pypi.org/pypi/onnxruntime/json)).

5. **Binary-only resolve on 3.14/win_amd64 succeeded.** Dry run: `py -3.14 -m pip install --dry-run --only-binary=:all: --python-version 3.14 --platform win_amd64` for fastapi, uvicorn, qdrant-client, fastembed, ollama, httpx, biopython, pymupdf, pypdf, pdfplumber, bm25s, rank-bm25, onnxruntime, sentence-transformers, torch. Result: exit 0, 84 packages resolved, torch `2.15.0+cpu`, onnxruntime `1.31.1`, sentence-transformers `6.3.0`, fastembed `0.9.1`. This is a resolver check, not an install and not an import test.

6. **Qdrant hybrid fusion is server-side.** "Qdrant supports this through the Query API" and "Its main building block is the prefetch parameter, which lets a query run sub-requests." Fusion: "Reciprocal Rank Fusion (RRF) RRF considers the positions of results within each query..."; DBSF is "Available as of v1.11.0". The docs also note BM25 full-text search on payload fields "that have a full-text index." > quotes above, [Qdrant hybrid queries](https://qdrant.tech/documentation/search/hybrid-queries/), "Reciprocal Rank Fusion (RRF)" and "Distribution-Based Score Fusion (DBSF)" sections.

7. **Qdrant Windows binary exists.** Latest release tag `v1.20.0`, published 2026-10-22, includes asset `qdrant-x86_64-pc-windows-msvc.zip` ([GitHub API, qdrant/qdrant releases/latest](https://api.github.com/repos/qdrant/qdrant/releases/latest)). Client `qdrant-client` 1.20.0 is Apache-2.0 and "Local mode - use same API without running server" ([qdrant-client README](https://github.com/qdrant/qdrant-client)).


8. **PyMuPDF license.** PyPI license field: "Dual Licensed - GNU AFFERO GPL 3.0 or Artifex Commercial License" ([pymupdf PyPI JSON](https://pypi.org/pypi/pymupdf/json)). AGPL obligations attach when you distribute the software or offer it over a network; a thesis prototype run locally is usually not that, but confirm with the supervisor/university. Its cp311-abi3 Windows wheel works on 3.14.
   - pypdf's LICENSE is BSD-style: "Redistribution and use in source and binary forms, with or without" ([pypdf LICENSE](https://github.com/py-pdf/pypdf/blob/main/LICENSE)). pdfplumber is MIT ([pdfplumber PyPI JSON](https://pypi.org/pypi/pdfplumber/json), classifier).
   - Extraction quality on scientific PDFs was **not** tested here (see Gaps).

9. **BM25 choice.** fastembed's `Bm25` produces sparse vectors for Qdrant, so lexical search uses the same Query API as dense. `rank-bm25` 0.2.2 was last uploaded 2022-02-16 ([rank-bm25 PyPI JSON](https://pypi.org/pypi/rank-bm25/json)), so avoid it. `bm25s` 0.3.16 is MIT ([bm25s PyPI JSON](https://pypi.org/pypi/bm25s/json)) and is the in-Python alternative.

10. **Ollama client.** The README says "pip install ollama" and "A custom client can be created by instantiating `Client` or `AsyncClient` from `ollama`." ([ollama-python README](https://github.com/ollama/ollama-python)). Use the client; plain httpx works too but duplicates it.

11. **PubMed rate limits.** Without a key: "you only need to get one if you expect to access E-utilities at a rate of more than three requests per second from a single computer (IP address)." With a key: "By default, your key will increase the limit to 10 requests/second" ([NCBI Insights, New API Keys for the E-utilities](https://ncbiinsights.ncbi.nlm.nih.gov/2017/11/02/new-api-keys-for-the-e-utilities/)). Biopython 1.88 has cp314 win_amd64 wheels ([biopython PyPI JSON](https://pypi.org/pypi/biopython/json)), but plain httpx calls to E-utilities are enough for this project.

12. **Python management on Windows.** Python 3.13 is already installed (`py -0p` lists `-V:3.13` at `...\Python313\python.exe`). `py -3.13 -m venv .venv` is the fallback. uv (0.13.7, has a `py3-none-win_amd64` wheel, [uv PyPI JSON](https://pypi.org/pypi/uv/json)) can also install Python versions: "uv bundles a list of downloadable CPython and PyPy distributions for macOS, Linux, and Windows." ([uv docs, Python versions](https://docs.astral.sh/uv/concepts/python-versions/)). Not required; venv + pip is simpler for this project.

## Comparisons

| Choice | Option A (recommended) | Option B | Trade-off |
|---|---|---|---|
| Embeddings/reranker | fastembed (ONNX) | sentence-transformers + torch CPU | A avoids torch; B is more common in tutorials but has a much larger download. |
| Hybrid fusion | Qdrant Query API, RRF server-side | RRF in Python | A: less code, one place for logic; B: more code, easier to inspect. |
| BM25 | fastembed `Bm25` → Qdrant sparse | bm25s in memory | A stays inside Qdrant; B needs a second index kept in sync. |
| PDF | pypdf (BSD) | PyMuPDF (AGPL/commercial) | Quality on scientific PDFs unverified; decide after a test on 3–5 real papers. |
| PubMed | httpx | Biopython Entrez | Few endpoints; httpx avoids a dependency with a large API surface. |

## Gaps

- **PDF extraction quality not measured.** Test pypdf vs PyMuPDF on 3–5 real papers from the corpus (multi-column, tables, references) before choosing.
- **Import/runtime test not run.** The 3.14 verdict rests on a resolver dry run. Next step: `py -3.14 -m venv .venv`, install the list above, then `python -c "import fastembed, qdrant_client, fastapi"` and load bge-small once.
- **fastembed `Bm25` vs Qdrant server BM25 not compared.** Only the fastembed class was read; check its output shape against the Qdrant sparse-vector docs before wiring.
- **Biopython vs httpx:** no primary doc for Biopython Entrez was fetched.
- **NCBI E-utilities docs page** (`/books/NBK25497/`) is JS-rendered and was not fetched; the NCBI blog is the cited source for rate limits.
- **Python 3.14 free-threaded (`cp314t`) wheels** exist for torch but were not chosen; use the standard `cp314` build.
