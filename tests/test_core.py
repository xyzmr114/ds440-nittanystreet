"""Cross checks for the internal Black Scholes module against py_vollib.

This is the audit gate from the spec: internal BSM must match py_vollib
to 6 decimals on 100 random contracts.
"""
import random

import pytest
from py_vollib.black_scholes import black_scholes as pv_price

from src.models.bsm import (
    black_scholes_delta,
    black_scholes_gamma,
    black_scholes_price,
)

random.seed(42)
CASES = [
    (random.uniform(20, 200), random.uniform(20, 200), random.uniform(0.01, 1.0),
     random.uniform(0.0, 0.05), random.uniform(0.1, 0.8), random.choice(["c", "p"]))
    for _ in range(100)
]


@pytest.mark.parametrize("S,K,T,r,sigma,cp", CASES)
def test_price_matches_py_vollib(S, K, T, r, sigma, cp):
    ours = black_scholes_price(S, K, T, r, sigma, cp)
    theirs = pv_price(cp, S, K, T, r, sigma)
    assert ours == pytest.approx(theirs, abs=1e-6)


def test_delta_bounds():
    # Delta is bounded by [0, 1] for calls and [-1, 0] for puts.
    for _ in range(20):
        S = random.uniform(50, 150)
        K = random.uniform(50, 150)
        T = random.uniform(0.01, 1.0)
        sigma = random.uniform(0.1, 0.8)
        assert 0.0 <= black_scholes_delta(S, K, T, 0.02, sigma, "c") <= 1.0
        assert -1.0 <= black_scholes_delta(S, K, T, 0.02, sigma, "p") <= 0.0


def test_gamma_positive():
    S, K, T, sigma = 100.0, 100.0, 0.5, 0.3
    assert black_scholes_gamma(S, K, T, 0.02, sigma) > 0.0


def test_call_put_parity():
    # C - P = S - K e^{-rT}
    S, K, T, r, sigma = 100.0, 105.0, 0.5, 0.02, 0.25
    import math
    c = black_scholes_price(S, K, T, r, sigma, "c")
    p = black_scholes_price(S, K, T, r, sigma, "p")
    assert c - p == pytest.approx(S - K * math.exp(-r * T), abs=1e-8)


def test_impact_square_root_shape():
    from src.models.impact import permanent_impact, temporary_impact

    # Free alpha greater than 0.5 means convex cost growth.
    base = temporary_impact(0.1, 1.0, 1.0, 0.5)
    doubled = temporary_impact(0.2, 1.0, 1.0, 0.5)
    assert doubled == pytest.approx(base * (2 ** 0.5), rel=1e-9)

    perm = permanent_impact(50000, 1000000, 0.1)
    assert perm == pytest.approx(0.005, rel=1e-9)


def test_index_move_arithmetic():
    from src.models.omega import index_move

    moves = {"JPM": 0.002, "BAC": 0.001, "WFC": -0.0005}
    weights = {"JPM": 0.08, "BAC": 0.05, "WFC": 0.04}
    expected = 0.08 * 0.002 + 0.05 * 0.001 + 0.04 * (-0.0005)
    assert index_move(moves, weights) == pytest.approx(expected, rel=1e-9)


def test_twap_sums_to_order():
    import numpy as np
    from src.models.scheduler import twap_schedule

    slices = twap_schedule(order_size=1000.0, horizon=1.0, n_slices=10)
    assert np.isclose(slices.sum(), 1000.0)
    assert np.allclose(slices, 100.0)


def test_vwap_follows_curve():
    import numpy as np
    from src.models.scheduler import vwap_schedule

    curve = np.array([1, 2, 4, 8, 4, 2, 1], dtype=float)  # U shape
    slices = vwap_schedule(curve, order_size=220.0, n_slices=7)
    assert np.isclose(slices.sum(), 220.0)
    assert slices.argmax() == 3
