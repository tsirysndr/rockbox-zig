"""Global user settings (EQ, replaygain, crossfade, …)."""

from __future__ import annotations

from typing import Any

from ..transport import HttpTransport
from ..types import UserSettings

_SETTINGS_FIELDS = """
musicDir volume balance bass treble channelConfig stereoWidth
eqEnabled eqPrecut
eqBandSettings { cutoff q gain }
replaygainSettings { noclip type preamp }
compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
crossfeedHfAttenuation crossfeedHfCutoff
repeatMode singleMode partyMode shuffle playerName
"""


class SettingsApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def get(self) -> UserSettings:
        data = await self._http.execute(
            f"query GlobalSettings {{ globalSettings {{ {_SETTINGS_FIELDS} }} }}"
        )
        return UserSettings.model_validate(data["globalSettings"])

    async def save(self, settings: dict[str, Any] | UserSettings) -> None:
        """Apply a partial update to the global settings.

        Accepts either a dict (snake_case keys are converted to camelCase) or a
        full ``UserSettings`` instance.
        """
        if isinstance(settings, UserSettings):
            payload = settings.model_dump(by_alias=True, exclude_none=True)
        else:
            payload = _to_camel_keys(settings)
        await self._http.execute(
            "mutation SaveSettings($settings: NewGlobalSettings!) "
            "{ saveSettings(settings: $settings) }",
            {"settings": payload},
        )


def _to_camel_keys(d: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for k, v in d.items():
        parts = k.split("_")
        camel = parts[0] + "".join(p.title() for p in parts[1:]) if "_" in k else k
        if isinstance(v, dict):
            v = _to_camel_keys(v)
        elif isinstance(v, list):
            v = [_to_camel_keys(x) if isinstance(x, dict) else x for x in v]
        out[camel] = v
    return out
