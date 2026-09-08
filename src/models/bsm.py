"""Black Scholes pricing and Greeks. Internal implementation is the source
of truth for the report; tests cross check against py_vollib to 6 decimals."""
from math import exp, log, sqrt

from scipy.stats import norm


def _d1(S: float, K: float, T: float, r: float, sigma: float) -> float:
    if T <= 0 or sigma <= 0:
        raise ValueError("T and sigma must be positive")
    return (log(S / K) + (r + 0.5 * sigma**2) * T) / (sigma * sqrt(T))


def _d2(d1: float, T: float, sigma: float) -> float:
    return d1 - sigma * sqrt(T)


def black_scholes_price(S: float, K: float, T: float, r: float, sigma: float,
                        cp_flag: str) -> float:
    """Price a European option. cp_flag is 'c' or 'p'."""
    if cp_flag not in ("c", "p"):
        raise ValueError("cp_flag must be 'c' or 'p'")
    d1 = _d1(S, K, T, r, sigma)
    d2 = _d2(d1, T, sigma)
    if cp_flag == "c":
        return S * norm.cdf(d1) - K * exp(-r * T) * norm.cdf(d2)
    return K * exp(-r * T) * norm.cdf(-d2) - S * norm.cdf(-d1)


def black_scholes_delta(S: float, K: float, T: float, r: float, sigma: float,
                        cp_flag: str) -> float:
    if cp_flag not in ("c", "p"):
        raise ValueError("cp_flag must be 'c' or 'p'")
    d1 = _d1(S, K, T, r, sigma)
    return norm.cdf(d1) if cp_flag == "c" else norm.cdf(d1) - 1.0


def black_scholes_gamma(S: float, K: float, T: float, r: float, sigma: float) -> float:
    d1 = _d1(S, K, T, r, sigma)
    return norm.pdf(d1) / (S * sigma * sqrt(T))
