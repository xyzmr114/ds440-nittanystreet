"""Almgren Chriss market impact model.

Temporary impact: I_temp = eta * (v_t / V_ADV) ** alpha
Permanent impact: I_perm = gamma * (Q / V_ADV)

Calibration fits eta, alpha, gamma on a time window and returns values
with standard errors. The square root law (alpha = 0.5) is tested against
a free alpha and both are reported.
"""
from dataclasses import dataclass


@dataclass
class ImpactParams:
    eta: float
    alpha: float
    gamma: float
    eta_se: float
    alpha_se: float
    gamma_se: float
    window: str
    n_obs: int


def temporary_impact(trading_rate: float, adv: float, eta: float, alpha: float) -> float:
    """Temporary impact at an instantaneous trading rate."""
    if adv <= 0:
        raise ValueError("adv must be positive")
    return eta * (trading_rate / adv) ** alpha


def permanent_impact(order_size: float, adv: float, gamma: float) -> float:
    """Permanent impact of a parent order of size Q against ADV."""
    if adv <= 0:
        raise ValueError("adv must be positive")
    return gamma * (order_size / adv)


def calibrate_impact(bars, depth, window: str) -> ImpactParams:
    """Fit eta, alpha, gamma on the given window. Returns point estimates
    plus standard errors. Implementation lands in Sprint 2."""
    raise NotImplementedError("Sprint 2: calibrate on one minute bars plus OFI")
