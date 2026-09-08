"""Execution schedulers (stretch goal).

TWAP: flat slices. VWAP: slices proportional to the U shaped volume
curve. Risk averse Almgren Chriss: closed form calculus of variations
trajectory (kappa tanh solution from the 2000 paper).
"""


def twap_schedule(order_size: float, horizon: float, n_slices: int):
    """Flat slices over the horizon."""
    if n_slices <= 0:
        raise ValueError("n_slices must be positive")
    import numpy as np
    return np.full(n_slices, order_size / n_slices)


def vwap_schedule(volume_curve, order_size: float, n_slices: int):
    """Slices proportional to the intraday volume curve."""
    if n_slices <= 0:
        raise ValueError("n_slices must be positive")
    import numpy as np
    curve = np.asarray(volume_curve, dtype=float)
    if curve.sum() <= 0:
        raise ValueError("volume curve must have positive mass")
    return order_size * curve / curve.sum()


def ac_optimal_schedule(order_size, horizon, eta, gamma, risk_aversion, volatility):
    """Risk averse Almgren Chriss optimal trajectory. Sprint 5."""
    raise NotImplementedError("Sprint 5: kappa tanh closed form solution")
