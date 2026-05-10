const { withAndroidManifest, withDangerousMod } = require("@expo/config-plugins");
const fs = require("fs");
const path = require("path");

// Allow cleartext (HTTP) traffic for all domains — needed for album art served
// over plain HTTP from rockboxd or other LAN sources. In dev builds,
// expo-dev-client injects its own permissive security config; in release builds
// that config is absent and Android's default blocks cleartext, which silently
// overrides the `usesCleartextTraffic` manifest flag whenever any
// networkSecurityConfig attribute is present in the merged manifest.
const NETWORK_SECURITY_CONFIG = `<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
  <base-config cleartextTrafficPermitted="true">
    <trust-anchors>
      <certificates src="system" />
    </trust-anchors>
  </base-config>
</network-security-config>
`;

const withNetworkSecurityConfig = (config) => {
  config = withDangerousMod(config, [
    "android",
    (config) => {
      const xmlDir = path.join(
        config.modRequest.platformProjectRoot,
        "app/src/main/res/xml"
      );
      fs.mkdirSync(xmlDir, { recursive: true });
      fs.writeFileSync(
        path.join(xmlDir, "network_security_config.xml"),
        NETWORK_SECURITY_CONFIG
      );
      return config;
    },
  ]);

  config = withAndroidManifest(config, (config) => {
    const app = config.modResults.manifest.application[0];
    app.$["android:networkSecurityConfig"] = "@xml/network_security_config";
    return config;
  });

  return config;
};

module.exports = withNetworkSecurityConfig;
