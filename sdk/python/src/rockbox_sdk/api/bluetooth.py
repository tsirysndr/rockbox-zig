"""Bluetooth pairing and discovery (Linux-only on the server)."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import BluetoothDevice
from ._fragments import BLUETOOTH_DEVICE_FIELDS


class BluetoothApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def devices(self) -> list[BluetoothDevice]:
        """List paired/known Bluetooth devices."""
        data = await self._http.execute(
            f"{BLUETOOTH_DEVICE_FIELDS} "
            "query BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }"
        )
        return [BluetoothDevice.model_validate(d) for d in data.get("bluetoothDevices", [])]

    async def scan(self, timeout_secs: int | None = None) -> list[BluetoothDevice]:
        """Trigger a scan for nearby devices and return discoveries."""
        data = await self._http.execute(
            f"{BLUETOOTH_DEVICE_FIELDS} "
            "mutation BluetoothScan($timeoutSecs: Int) "
            "{ bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields } }",
            {"timeoutSecs": timeout_secs},
        )
        return [BluetoothDevice.model_validate(d) for d in data.get("bluetoothScan", [])]

    async def connect(self, address: str) -> None:
        await self._http.execute(
            "mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }",
            {"address": address},
        )

    async def disconnect(self, address: str) -> None:
        await self._http.execute(
            "mutation BluetoothDisconnect($address: String!) "
            "{ bluetoothDisconnect(address: $address) }",
            {"address": address},
        )
