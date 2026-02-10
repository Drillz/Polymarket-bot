## 2024-11-05 - String Allocations in Hot Paths
**Learning:** Pre-computing `extract_entities` and storing in `Market` struct removed O(N*M) string allocations in the dependency analysis loop.
**Action:** Always check if expensive derived data (like entities or tokens) can be computed once during normalization/ingestion.
