from __future__ import annotations

from typing import Any

import httpx


class PennyKiteError(Exception):
    """Base exception for PennyKite client errors."""


class PennyKiteDenied(PennyKiteError):
    def __init__(
        self,
        *,
        verdict: str,
        reason: str,
        response: httpx.Response,
        decision_id: str | None = None,
        decision_hash: str | None = None,
        estimated_cost_usd: float | None = None,
        payload: dict[str, Any] | None = None,
    ) -> None:
        self.verdict = verdict
        self.reason = reason
        self.response = response
        self.decision_id = decision_id
        self.decision_hash = decision_hash
        self.estimated_cost_usd = estimated_cost_usd
        self.payload = payload or {}
        super().__init__(f"{verdict}: {reason}")


class PennyKiteBudgetExceeded(PennyKiteDenied):
    pass


class PennyKiteLoopDetected(PennyKiteDenied):
    pass
