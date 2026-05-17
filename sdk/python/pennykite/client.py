from __future__ import annotations

from collections.abc import Mapping
from typing import Any
from urllib.parse import urlsplit

import httpx

from .exceptions import PennyKiteBudgetExceeded, PennyKiteDenied, PennyKiteLoopDetected

SESSION_HEADER = "x-pennykite-session"


class PennyKiteClient:
    def __init__(
        self,
        *,
        session_id: str,
        proxy_url: str = "http://127.0.0.1:8787",
        timeout: float | httpx.Timeout = 30.0,
        headers: Mapping[str, str] | None = None,
        transport: httpx.BaseTransport | None = None,
        http_client: httpx.Client | None = None,
    ) -> None:
        if not session_id:
            raise ValueError("session_id is required")
        self.session_id = session_id
        self.proxy_url = proxy_url.rstrip("/")
        self._headers = dict(headers or {})
        self._owns_client = http_client is None
        self._client = http_client or httpx.Client(
            base_url=self.proxy_url,
            timeout=timeout,
            transport=transport,
        )

    def __enter__(self) -> PennyKiteClient:
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        self.close()

    def close(self) -> None:
        if self._owns_client:
            self._client.close()

    def get(self, url: str, **kwargs: Any) -> httpx.Response:
        return self.request("GET", url, **kwargs)

    def post(self, url: str, **kwargs: Any) -> httpx.Response:
        return self.request("POST", url, **kwargs)

    def request(self, method: str, url: str, **kwargs: Any) -> httpx.Response:
        headers = dict(self._headers)
        headers.update(kwargs.pop("headers", {}) or {})
        headers[SESSION_HEADER] = self.session_id
        response = self._client.request(
            method,
            _proxy_target(url),
            headers=headers,
            **kwargs,
        )
        _raise_for_pennykite_denial(response)
        return response


def _proxy_target(url: str) -> str:
    parts = urlsplit(url)
    if parts.scheme and parts.netloc:
        path = parts.path or "/"
        return f"{path}?{parts.query}" if parts.query else path
    if url.startswith("/"):
        return url
    return f"/{url}"


def _raise_for_pennykite_denial(response: httpx.Response) -> None:
    if response.status_code != 402:
        return
    try:
        payload = response.json()
    except ValueError:
        return
    if not isinstance(payload, dict) or "verdict" not in payload:
        return

    verdict = str(payload.get("verdict") or "deny")
    reason = str(payload.get("reason") or "PennyKite denied the request")
    kwargs = {
        "verdict": verdict,
        "reason": reason,
        "response": response,
        "decision_id": _optional_str(payload.get("decision_id")),
        "decision_hash": _optional_str(payload.get("decision_hash")),
        "estimated_cost_usd": _optional_float(payload.get("estimated_cost_usd")),
        "payload": payload,
    }
    if verdict == "deny_loop":
        raise PennyKiteLoopDetected(**kwargs)
    if verdict == "deny_budget":
        raise PennyKiteBudgetExceeded(**kwargs)
    raise PennyKiteDenied(**kwargs)


def _optional_str(value: Any) -> str | None:
    return str(value) if value is not None else None


def _optional_float(value: Any) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None
