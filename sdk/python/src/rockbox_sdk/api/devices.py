"""Cast / source device discovery and control."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import Device

_FIELDS = (
    "id name host ip port service app isConnected "
    "baseUrl isCastDevice isSourceDevice isCurrentDevice"
)


class DevicesApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def list(self) -> list[Device]:
        data = await self._http.execute(
            f"query Devices {{ devices {{ {_FIELDS} }} }}"
        )
        return [Device.model_validate(d) for d in data.get("devices", [])]

    async def get(self, id: str) -> Device | None:
        data = await self._http.execute(
            f"query Device($id: String!) {{ device(id: $id) {{ {_FIELDS} }} }}",
            {"id": id},
        )
        raw = data.get("device")
        return Device.model_validate(raw) if raw is not None else None

    async def connect(self, id: str) -> None:
        await self._http.execute(
            "mutation ConnectDevice($id: String!) { connect(id: $id) }", {"id": id}
        )

    async def disconnect(self, id: str) -> None:
        await self._http.execute(
            "mutation DisconnectDevice($id: String!) { disconnect(id: $id) }", {"id": id}
        )
