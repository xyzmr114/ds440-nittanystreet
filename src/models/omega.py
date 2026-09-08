"""Cross asset propagation matrix.

Delta P_index(t) = sum_i w_i(t) * Delta P_i(t)

Exact index arithmetic. The research question is the weights: official,
float adjusted, and volume weighted variants are compared.
"""


def index_move(constituent_moves: dict[str, float],
               weights: dict[str, float]) -> float:
    """Index level move from simulated constituent moves."""
    missing = set(weights) - set(constituent_moves)
    if missing:
        raise ValueError(f"missing constituent moves for: {missing}")
    return sum(weights[sym] * move for sym, move in constituent_moves.items())


def build_weight_matrix(holdings, variant: str):
    """Build a weight vector from a holdings frame.

    variant is one of: official, float_adjusted, volume_weighted.
    Implementation lands in Sprint 1 along with the holdings fetch.
    """
    raise NotImplementedError("Sprint 1: build weights from SPDR holdings files")
