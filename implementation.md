# Implementation Guide: Deterministic Polymarket Arbitrage Engine

This document outlines the implementation plan for a rule-based arbitrage detection system. Unlike the reference paper which uses LLMs for dependency detection, this system uses **deterministic heuristics** (Regex, Jaccard Similarity, and Subset Logic) to identify the arbitrage opportunities defined in *Saguillo et al., 2025*.

## 1. System Architecture

The system is composed of three deterministic Rust modules:
1.  **Market Ingestion & Normalization**: Fetches data and standardizes dates/tags.
2.  **Rule-Based Dependency Graph**: Replaces the "Probabilistic Forest" with a "Logic Graph". It links markets based on strict string matching and category rules.
3.  **Arbitrage Calculator**: Computes risk-free profit based on Definitions 3 & 4.

---

## 2. Implementation Steps

### Step 1: Data Normalization (The Foundation)
**Goal**: Ensure apples-to-apples comparison for strings and dates.
*   **Timestamp Alignment**: As per the paper, markets with the same `market_slug` or parent event must share an `end_date_iso`.
    *   *Rule*: Group by `neg_risk_market_id`. Force all conditions in the group to use the **latest** `end_date` found in the set.
*   **String Sanitization**:
    *   Convert all Questions and Condition names to lowercase.
    *   Remove stop words ("the", "will", "be", "outcome").
    *   Standardize separators (replace " - " with ":").

### Step 2: Search Space Reduction (Deterministic Clustering)
**Goal**: efficiently pair potential dependent markets without $O(N^2)$ scanning.
*   **Cluster by Tag**: Polymarket provides tags (e.g., "Politics", "Sports"). Only compare markets sharing at least one primary tag.
*   **Cluster by Date**: Only compare markets with identical resolved `end_date_iso`.
*   **Cluster by Entity**: Extract proper nouns (e.g., "Trump", "Bitcoin", "Lakers"). Two markets are candidates for dependency **only if** they share at least one Entity.

### Step 3: Rule-Based Dependency Detection
**Goal**: Identify if Market A implies Market B without an LLM.
*   **Heuristic A: The "Winner vs. Margin" Rule**
    *   *Detection*: Market A title contains "winner"; Market B title contains "margin".
    *   *Link*: If Market B condition is "By >5%" and Market A condition is "Candidate X", check if "Candidate X" is present in the "Winner" market.
*   **Heuristic B: The Subset String Rule**
    *   *Detection*: If Condition Name $C_a$ is a substring of Condition Name $C_b$ (or vice versa) AND they belong to the same Event Cluster.
    *   *Logic*: "Trump wins by >10%" (Subset) implies "Trump wins" (Superset).
*   **Heuristic C: Numerical Range Overlap**
    *   *Detection*: Parse numbers from strings (e.g., " > 5%", "10-20%").
    *   *Logic*: If Range A (5-10) falls strictly within Range B (0-100), then A implies B.

### Step 4: Execution Engine
**Goal**: Calculate profit.
*   **Rebalancing**: Sum prices within a single market ID. If $
e 1.0$, flag opportunity.
*   **Combinatorial**: For linked pairs (from Step 3), check if $Price(Subset) > Price(Superset)$. If true, Short Subset / Buy Superset.

---

## 3. Coding Agent Prompt

**Copy and paste this prompt to a coding agent to generate the Rust code:**

> "You are a Rust systems engineer. Implement a **rule-based** arbitrage detection engine for Polymarket.
>
> **Core Requirements:**
> 1.  **Crates**: Use `rust_decimal` for math, `regex` for parsing, and `strsim` for string similarity.
> 2.  **Structs**: Define `Market`, `Condition`, and `DependencyGraph`.
> 3.  **Module 1 (Rebalancing)**: Implement `check_rebalancing(market)`. Sum the prices. If `sum < 0.98` or `sum > 1.02` (accounting for fees), return a `RebalancingOpportunity` struct.
> 4.  **Module 2 (Dependency Logic)**:
>     *   Implement `are_markets_related(m1, m2) -> bool`: Returns true only if they share the same `end_date` AND have Jaccard Similarity > 0.3 on their titles.
>     *   Implement `detect_implication(c1, c2) -> Option<Direction>`:
>         *   **Rule 1**: If `c1.name` contains `c2.name` (e.g., 'Trump >5%' contains 'Trump'), then c1 implies c2.
>         *   **Rule 2**: Use Regex to extract numeric ranges. If range of c1 is inside range of c2, c1 implies c2.
> 5.  **Module 3 (Combinatorial Scan)**:
>     *   Iterate through related markets.
>     *   If `c1 implies c2` AND `price(c1) > price(c2)`, return a `CombinatorialOpportunity` (Profit = `price(c1) - price(c2)`).
>
> **Constraint**: Do NOT use LLMs or external APIs. The logic must be strictly deterministic and offline capable."