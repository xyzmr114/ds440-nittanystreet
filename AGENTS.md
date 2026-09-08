# DS 440: Cross Asset TCA Engine - Technical Spec for AI Agents

Version 1.0, Sep 8 2026. Group 2, Nittany Street.

This document is the single source of truth for every AI agent working on
this project (Claude Code, Codex, OpenCode, Cursor, whatever). Read it
cold, in full, before writing a line of code. If you find a contradiction
between this doc and the proposal PDF, this doc wins unless Harsh says
otherwise.

## 1. The project in one paragraph

A desk buys a basket of index constituents. The buying moves the stocks,
which moves the index, which revalues the desk's index options book. We
build an engine that measures whether the derivative leg gain beats the
cash leg execution cost, and at what trade size the edge dies. Three
stages: Almgren Chriss market impact per stock, a weight matrix that
propagates stock moves into the index, and a Black Scholes delta plus
gamma revaluation of an options book. Net PnL equals derivative gain
minus cash cost.

## 2. Ground rules for every agent

1. Never fabricate data. If a vendor fetch fails, log it and report the
   gap. No synthetic bars, no made up options chains.
2. Every number in any report must come from a script in the repo. No
   hand typed numbers in markdown.
3. Reproducibility is graded. Pin Python 3.12, use a lockfile, set a
   global seed, log all parameters. Another agent must be able to rerun
   your script and get the same numbers.
4. No hardcoded symbols. XLF and Bank Nifty are both generic weight
   matrices. A config file defines the universe.
5. Time based train and test split only. Never split by symbol. Cross
   sectional contamination inflates edge and the prof will catch it.
6. Write pytest tests for every public function. Minimum 2 tests per
   function. CI runs on every push.
7. Calibration outputs are ranges, not point estimates. Report the
   parameter plus its standard error plus the calibration window.
8. Style: type hints everywhere, docstrings on public functions, module
   contracts exactly as specified below. No clever tricks, readable code.
9. Data artifacts go in data/raw (untouched vendor files) and
   data/processed (Parquet). Nothing raw in git, only scripts.

## 3. Repository layout

```
ds440-tca/
  README.md
  AGENTS.md               (this file)
  pyproject.toml          (Python 3.12, dependencies pinned)
  config/
    universe.yaml         (symbols, index, weights source)
    data_sources.yaml     (vendor paths, keys via env vars)
  src/
    ingestion/
      fetch_bars.py
      fetch_weights.py
      fetch_options.py
      clean_bars.py
    features/
      adv.py              (ADV curves, U shape)
      volatility.py       (Parkinson)
      ofi.py              (order flow imbalance from depth)
    models/
      impact.py           (Almgren Chriss)
      omega.py            (propagation matrix)
      revalue.py          (Black Scholes book revaluation)
      scheduler.py        (TWAP, VWAP, risk averse AC, stretch)
    report/
      tca_report.py
      sweep.py            (scale sensitivity, stretch)
  data/
    raw/
    processed/
  tests/
  dashboard/
    app.py                (Streamlit, stretch)
```

## 4. Environment

Python 3.12. Dependencies: polars, duckdb, numpy, scipy, pyarrow,
py_vollib (or an internal Black Scholes module in src/models/bsm.py),
streamlit, pytest, python-dotenv. Use uv for the lockfile. API keys go in
a .env file that is gitignored. Never commit keys.

```bash
uv sync
uv run pytest
```

## 5. Data acquisition: exactly what we get and how

Owner per dataset is the Data and Modeling Lead (Ammar), but any agent
can implement the fetchers. All fetchers write to data/raw/ first.

### 5.1 Constituent equities: one minute OHLCV plus VWAP bars

What: 15 XLF constituents, 2 years, 1 minute bars. About 3 million rows,
under 500 MB in Parquet.

Primary: Databento academic tier (free for students). Steps:
1. Sign up at databento.com with the .edu email.
2. Request academic access, usually approved in days.
3. Use the databento-python SDK: DBEQ dataset for XLF constituents,
   1 minute OHLCV plus VWAP schema, last 2 years.
4. Export to Parquet with the SDK, no manual CSV conversion.

Fallbacks: Tiingo free tier (API key, limited 1m history, slower) or
Polygon.io free flat files (5 years of daily aggregates free, intraday
limited). Use fallbacks only for a subset of symbols if Databento stalls.

### 5.2 Index level bars

What: XLF 1 minute bars for the same 2 years, tiny footprint. Same vendor
as 5.1. Bank Nifty 1 minute bars only if the extension case gets
activated, from the same vendor or NSE data partners.

### 5.3 Limit order book depth for OFI calibration

What: LOBSTER or Databento MBO tick data. We sample selected days, not
full history. Ten symbols for one day is 10 to 40 GB.

Primary: LOBSTER (free for academic research). Steps:
1. Academic email required plus a short usage agreement.
2. Request specific dates for the 15 symbols (pick 10 to 15 sample days
   spread across regimes: calm, earnings season, high vol).
3. Parse message and orderbook files into a compact Parquet with columns:
   timestamp, price, size, side, event_type.

Fallback: Databento MBO with the academic tier (fewer free credits, so
sample fewer days).

The OFI feature (order flow imbalance) is computed from this depth data:
sum of buy limit insertions minus sell limit insertions plus
modifications per minute, normalized by volume. This feeds the impact
calibration as a regime control variable.

### 5.4 Index options chains and implied vols

What: daily chain snapshots for XLF options, 2 years of near term
expiries, implied vols, Greeks. A few GB.

Primary: ORATS free academic tier (has implied vol surfaces and Greeks
already computed). Steps:
1. Apply at orats.com for academic access with .edu email.
2. Pull daily snapshots of XLF chains: strike, expiry, IV, delta, gamma,
   volume, OI.
3. Store as Parquet: date, expiry, strike, cp_flag, iv, delta, gamma,
   bid, ask, oi.

Fallbacks: CBOE DataShop (registration, per dataset fees, some free
samples) or Databento OPRA (credit heavy, last resort).

Pitfall: historical chains have holes. Interpolate IV surfaces with light
smoothing. Flag every trade whose expiry sits in a gap and exclude it
from headline results.

### 5.5 Constituent weights

What: daily XLF holdings from the SPDR website (Excel downloads). Daily,
negligible footprint. For Bank Nifty: NSE published weights. Store as
Parquet: date, symbol, weight, shares_outstanding, float_factor if
available.

The propagation matrix uses three weight variants: official, float
adjusted, volume weighted. All three come from the same holdings file
plus volume data. This comparison is a named finding in the report.

### 5.6 Data quality rules

- Drop zero or negative prices, pre and post session rows.
- Adjust for splits and dividends before any return math.
- Reconstruct historical membership from holdings history. Never use
  today's constituent list for a 2024 backtest (survivorship bias).
- Store cleaned data in data/processed/ as Parquet with DuckDB views.
- Every fetch script logs source, date range, and row count to a
  manifest.json in data/processed/.

## 6. Module contracts

### 6.1 src/models/impact.py

```python
def temporary_impact(trading_rate: float, adv: float,
                     eta: float, alpha: float) -> float:
    """I_temp = eta * (trading_rate / adv) ** alpha"""

def permanent_impact(order_size: float, adv: float,
                     gamma: float) -> float:
    """I_perm = gamma * (order_size / adv)"""

def calibrate_impact(bars, depth, window: str) -> ImpactParams:
    """Fit eta, alpha, gamma on the calibration window. Return dataclass
    with values plus standard errors. Test alpha against 0.5 (square
    root law) and report both."""
```

Cross validation requirement: fit the Kyle linear model and the Gatheral
square root law on the same data. The report must show all three side by
side, not just the one we picked.

### 6.2 src/models/omega.py

```python
def index_move(constituent_moves: dict[str, float],
               weights: dict[str, float]) -> float:
    """Delta P index = sum(w_i * Delta P_i). Exact index arithmetic."""

def build_weight_matrix(holdings, variant: str) -> pd.DataFrame:
    """variant in {official, float_adjusted, volume_weighted}"""
```

### 6.3 src/models/revalue.py

```python
def revalue_book(index_move: float, book: OptionsBook,
                 vol_surface: VolSurface) -> BookPnl:
    """Reprice each option with Black Scholes delta and gamma at the new
    index level. First pass holds vol fixed, second pass moves IV with
    the index (smirk). Return delta_pnl, gamma_pnl, total."""

def black_scholes_price(S, K, T, r, sigma, cp_flag) -> float
def black_scholes_delta(S, K, T, r, sigma, cp_flag) -> float
def black_scholes_gamma(S, K, T, r, sigma) -> float
```

Audit requirement: internal Black Scholes must match py_vollib outputs
to 6 decimals on 100 random contracts. Keep the internal one as the
source of truth for the report (auditable).

### 6.4 src/report/tca_report.py

```python
def run_trade(trade: TradeSpec) -> TradeResult:
    """Full pipeline for one parent order: impact per constituent,
    index move, book revaluation, net PnL."""

def implementation_shortfall_bps(exec_prices, arrival_price) -> float
def vwap_slippage_bps(exec_prices, vwap) -> float
```

Metrics per trade and aggregate: implementation shortfall bps vs arrival,
VWAP slippage bps, net PnL, participation rate.

## 7. The four formulas (memorize, they are the whole project)

1. Temporary impact: I_temp(t) = eta * (v_t / V_ADV) ^ alpha
   v_t is trading rate at time t, V_ADV average daily volume. eta scale,
   alpha shape, alpha near 0.5 is the square root law.
2. Permanent impact: I_perm(t) = gamma * (Q / V_ADV)
   Q is total order size. Information leakage, does not revert.
3. Propagation: Delta P_index(t) = sum_i w_i(t) * Delta P_i(t)
   Exact index arithmetic. The research question is the weights.
4. Net PnL = Delta(derivative book value) - total cash leg execution cost
   Positive means the execution bought more on the options leg than it
   cost on the stock leg.

## 8. Sprint map for agents

- Sprint 1 (weeks 1 to 2): fetch and clean bars (5.1, 5.2), fetch
  holdings (5.5), ADV and volume curve features. Deliverable: data
  manifest plus Progress Report 1.
- Sprint 2 (weeks 3 to 4): calibrate impact on single stocks. Deliverable
  for this sprint: calibrated eta, alpha, gamma with standard errors, and
  the square root law test result.
- Sprint 3 (weeks 5 to 6): omega.py plus options data (5.4) plus
  revalue.py. Deliverable: first cross asset elasticity numbers.
- Sprint 4 (weeks 7 to 8): integrate legs, run the full pipeline on a
  trade list, start the sweep. Deliverable: full TCA simulation.
- Sprint 5 (weeks 9 to 10): stretch goal, section 9 below.
- Sprint 6 (weeks 11 to 12): ablations, error analysis, limitations.
  Final report and oral.

Definition of done for any sprint: tests pass, numbers reproducible from
scratch, progress report section written, artifacts in Parquet.

## 9. Stretch goal: full specification

The stretch goal has three pieces. They are only worked after Phase 1
core engine is green.

### 9.1 Execution scheduler (src/models/scheduler.py)

```python
def twap_schedule(order_size: float, horizon: float,
                  n_slices: int) -> np.ndarray
def vwap_schedule(volume_curve, order_size, n_slices) -> np.ndarray
def ac_optimal_schedule(order_size, horizon, eta, gamma,
                        risk_aversion, volatility) -> np.ndarray
```

TWAP: flat slices. VWAP: slices proportional to the U shaped intraday
volume curve. Risk averse Almgren Chriss: solve the calculus of
variations objective (speed vs impact vs risk tradeoff), closed form
trajectory, the kappa tanh solution from the 2000 paper.

Acceptance: given the same parent order, run all three schedules through
run_trade and report the cost difference. The AC schedule must beat TWAP
cost on average across the trade list when risk aversion is tuned, and
the report explains when it does not (low risk aversion collapses to
VWAP, which is a named sanity check).

### 9.2 Scale sensitivity sweep (src/report/sweep.py)

Grid: participation rates 1%, 5%, 10%, 25%, 50%, 75%, 100%, 125%, 150%,
200% of ADV. For each point: full pipeline net PnL per basket.

Output: the edge curve, PnL vs participation, and the crossing point
where net PnL turns negative. That crossing point is the "capital
distortion gate". The report states it as the headline finding.

Acceptance: sweep runs end to end in under 10 minutes on a laptop, curve
is reproducible, and the gate is quoted with the calibration range, not
as a point.

### 9.3 Dashboard (dashboard/app.py, Streamlit)

Inputs: basket choice (XLF or Bank Nifty), trade size as percent of ADV,
schedule choice. Outputs: execution cost in bps, simulated index
displacement, net PnL. Used for the sponsor demo.

Acceptance: three inputs, three outputs, page renders in under 2 seconds,
all numbers from run_trade, zero hardcoded values in the UI.

## 10. Pitfalls that have already killed similar projects

1. Lookahead bias: using data that was not published at trade time.
   Holdings as of T, not as of today. Options quotes from the right day.
2. Survivorship bias: backtesting with today's 15 constituents over 2024
   when the index held different names then. Use historical holdings.
3. Vol surface gaps near expiry: interpolate, flag, exclude.
4. Regime drift: impact params from 2023 do not hold in 2026. Calibrate
   per quarter and report the window.
5. Double counting impact: temporary and permanent impact overlap if the
   benchmark is wrong. Benchmark to arrival price, state it in the code.
6. Free tier rate limits: cache everything in Parquet after first fetch.
   Never refetch in a loop.
7. Synthetic data temptation: never. The prof asks where data came from
   in the oral. Every fetch has a manifest entry.

## 11. References the report must engage

Almgren and Chriss 2000 (the framework). Kyle 1985 (linear permanent
impact). Gatheral 2010 (square root law, no dynamic arbitrage). Cont,
Kukanov, Stoikov 2014 (OFI). Black and Scholes 1973 (the revaluation).
Bouchaud et al 2004 (market response). Kissell 2013 (TCA practice). The
Jane Street Nifty 50 case (CNBC and Reuters, July 2025) is the regulatory
context for why measuring self created impact matters.
