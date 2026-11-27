# OpenScout Intelligence

A full-stack football scouting and match event analytics platform powered by **Hudl/StatsBomb Open Data**, **FastAPI**, **PostgreSQL**, and **Next.js**.

![Argentina 2022 World Cup Final xG Shot Map](docs/images/wc2022_final_argentina_shotmap.png)

## Architecture

- **Rust Core (`/backend-rust`)**: High-performance `Axum` + `Tokio` + `SQLx` REST API & StatsBomb JSON ETL engine backed by PostgreSQL 16 (hybrid relational + `JSONB` event schema).
- **Analytics Sidecar (`/backend`)**: Python `statsbombpy` + `mplsoccer` scripts for publication-grade pitch visualizations and ML experiments.
- **Frontend (`/frontend`)**: Next.js 15, TypeScript, Tailwind CSS (Milestone 4).

## Quick Start (Backend)

1. Start PostgreSQL:
   ```bash
   docker compose up -d
   ```
2. Install dependencies with `uv`:
   ```bash
   cd backend
   uv sync
   ```
3. Run the API:
   ```bash
   uv run uvicorn app.main:app --reload
   ```
