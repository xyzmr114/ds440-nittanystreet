# DS 440: Cross Asset TCA and Market Elasticity Engine

Group 2, Nittany Street. Penn State DS 440 Capstone, Fall 2026.

When a desk buys big positions in index constituents, its own buying
pressure lifts the stocks, moves the index, and revalues its index
options book. This project measures whether that derivative leg gain
beats the cash leg execution cost, and at what scale the edge turns into
price distortion.

## What this repo contains

- A market impact model (Almgren Chriss, temporary plus permanent)
  calibrated on one minute bars and order flow imbalance
- A propagation matrix that translates constituent moves into index moves
  through constituent weights
- A Black Scholes delta and gamma revaluation of an index options book
- A net PnL report: derivative gain minus cash cost, per trade and
  aggregate
- Stretch: execution schedulers (TWAP, VWAP, risk averse AC), a 1% to
  200% ADV scale sweep, and a Streamlit dashboard

## Team

| Role | Owner |
|---|---|
| Product Owner / Lead Author | Harsh |
| Scrum Master / Process Lead | Aryamaan |
| Data and Modeling Lead | Ammar |
| Quant Modeling Lead | Akshat |
| Evaluation and Infrastructure | Saathvik |

## Quick start

```bash
git clone https://github.com/xyzmr114/ds440-nittanystreet.git
cd ds440-nittanystreet
uv sync
uv run pytest
```

API keys live in a gitignored `.env` file. See `config/data_sources.yaml`
for the vendor map.

## For AI agents

Read `AGENTS.md` before writing any code. It is the technical spec and
the single source of truth for this project.

## Docs

- `AGENTS.md` — full technical specification for humans and agents
- `config/universe.yaml` — symbols, index, and weights source
- `config/data_sources.yaml` — vendor endpoints and fallbacks

## License

Public domain (Unlicense). See LICENSE.
