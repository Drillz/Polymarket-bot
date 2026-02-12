## 2024-05-22 - Market Normalization Guarantees
**Learning:** Market titles and condition names are already normalized (lowercase, sanitized) during ingestion.
**Action:** Avoid redundant `to_lowercase()` or sanitization in hot loops (like arbitrage checks). Trust the normalization pipeline.
