"""Exception hierarchy for the Rockbox SDK."""

from __future__ import annotations

from typing import Any


class RockboxError(Exception):
    """Base error for everything the SDK raises."""

    def __init__(self, message: str, cause: Exception | None = None) -> None:
        super().__init__(message)
        self.message = message
        self.cause = cause


class RockboxNetworkError(RockboxError):
    """Raised when the SDK cannot reach the rockboxd HTTP/WS endpoint."""


class RockboxGraphQLError(RockboxError):
    """Raised when the server returns a non-empty `errors` array."""

    def __init__(self, errors: list[dict[str, Any]]) -> None:
        self.errors = errors
        super().__init__("; ".join(str(e.get("message", e)) for e in errors))
