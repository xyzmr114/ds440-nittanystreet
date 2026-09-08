"""Options book revaluation on a simulated index move.

Net PnL = Delta(derivative book value) - total cash leg execution cost.

First pass holds the vol surface fixed, second pass lets IV move with the
index (the smirk matters at the money).
"""
from dataclasses import dataclass


@dataclass
class BookPnl:
    delta_pnl: float
    gamma_pnl: float
    vega_pnl: float
    total: float


def revalue_book(index_move: float, book, vol_surface) -> BookPnl:
    """Reprice every position with Black Scholes delta and gamma at the
    new index level. Implementation lands in Sprint 3."""
    raise NotImplementedError("Sprint 3: repricing with delta and gamma")
