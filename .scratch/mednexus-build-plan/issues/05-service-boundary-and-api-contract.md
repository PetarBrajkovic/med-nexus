Type: grilling
Status: open
Blocked by: 01, 02, 03

# Rust↔Python boundary and API contract

## Question

What exactly does the contract between the Rust API and the Python AI service look like? Who receives the upload and where is the file stored; who orchestrates ingest (does Rust call Python and wait, or does Python run a background job with a status Rust polls); which endpoints does Python expose (`/ingest`, `/search`, `/answer`, ...) and with what JSON shapes; does the frontend ever talk to Python directly (no?); how are errors and timeouts (an answer takes 30–60 s) handled?
