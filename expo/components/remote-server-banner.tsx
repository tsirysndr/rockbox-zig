import { router } from "expo-router";
import { Pressable, Text, View } from "react-native";

import { useIsRemoteServer } from "@/lib/connection";
import { useSelectedServer } from "@/lib/server-store";

/** Height of the banner row — consumers that need to offset absolute overlays can import this. */
export const REMOTE_BANNER_HEIGHT = 28;

/**
 * Inline banner row. Renders nothing when the server is local or not connected.
 * Callers are responsible for positioning (flow or absolute wrapper).
 */
export function RemoteServerBanner() {
  const isRemote = useIsRemoteServer();
  const server = useSelectedServer();

  if (!isRemote || !server) return null;

  return (
    <Pressable
      onPress={() => router.push("/settings/server")}
      style={{
        height: REMOTE_BANNER_HEIGHT,
        backgroundColor: "#39FF14",
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        gap: 6,
      }}
    >
      <View
        style={{
          width: 6,
          height: 6,
          borderRadius: 3,
          backgroundColor: "#000",
          opacity: 0.45,
        }}
      />
      <Text
        style={{
          color: "#000",
          fontSize: 11,
          fontFamily: "SpaceGrotesk",
          fontWeight: "700",
          letterSpacing: 0.4,
        }}
      >
        Controlling {server.label} · {server.host}
      </Text>
    </Pressable>
  );
}
