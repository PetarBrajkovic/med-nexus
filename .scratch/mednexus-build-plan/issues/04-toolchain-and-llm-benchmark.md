Type: task
Status: open
Blocked by: —

# Install toolchain and measure LLM speed

## Question

Install the missing tools and gather the facts the model-choice decision depends on. HITL: the human runs the commands, which doubles as their first contact with the toolchain.

- [ ] Install Rust via `rustup` (MSVC toolchain + Visual Studio Build Tools); check with `cargo --version`
- [ ] Install Ollama for Windows; `ollama --version`
- [ ] Download the Qdrant Windows binary, start it, open `http://localhost:6333/dashboard`
- [ ] Check that MySQL 8.4 is running and that you can log in
- [ ] `ollama pull` for 2–3 candidates (Llama 3.2 3B, Gemma 3 4B, Qwen 2.5 3B, quantized) and measure, with a ~3k-token prompt, **prompt eval tokens/s** and **generation tokens/s** (`ollama run <model> --verbose`)

Resolution: record the versions, the table of measurements, and anything that broke.
