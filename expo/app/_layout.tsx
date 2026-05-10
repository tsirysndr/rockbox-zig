import "../global.css";
import "@/lib/nativewind-setup";

// expo's withDevTools calls useKeepAwake() in dev builds. On Android, the brief
// window between Activity A destruction and Activity B initialization leaves
// activityProvider.currentActivity === null, so ExpoKeepAwakeManager.activate()
// throws CurrentActivityNotFoundException → unhandled rejection → red error
// overlay that blocks all interaction until dismissed. Suppress it here: the
// screen-keep-on re-activates harmlessly on the next render, and this code
// path is dead in production builds.
if (__DEV__) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const EU = (globalThis as any).ErrorUtils as
    | { getGlobalHandler(): (e: Error, f: boolean) => void; setGlobalHandler(h: (e: Error, f: boolean) => void): void }
    | undefined;
  if (EU) {
    const _prevEH = EU.getGlobalHandler();
    EU.setGlobalHandler((error: Error, isFatal: boolean) => {
      if (error?.message?.includes("Unable to activate keep awake")) return;
      _prevEH(error, isFatal);
    });
  }
}

import { ThemeProvider } from "@react-navigation/native";
import { QueryClient, QueryClientProvider, focusManager } from "@tanstack/react-query";
import { useFonts } from "expo-font";
import { Stack } from "expo-router";
import * as SplashScreen from "expo-splash-screen";
import { StatusBar } from "expo-status-bar";
import { useEffect, useRef, useState } from "react";
import { Alert, AppState, Platform } from "react-native";
import "react-native-reanimated";

// Wire TanStack Query's focus manager to React Native's AppState so that
// refetchOnWindowFocus (app foreground) works correctly on iOS / Android.
focusManager.setEventListener((handleFocus) => {
  const sub = AppState.addEventListener("change", (state) =>
    handleFocus(state === "active"),
  );
  return () => sub.remove();
});

import { PersistentMiniPlayer } from "@/components/persistent-mini-player";
import { TrackContextMenu } from "@/components/track-context-menu";
import { Colors } from "@/constants/theme";
import { PlayerProvider } from "@/lib/player-context";
import { RockboxClient } from "@/lib/rockbox-client";
import { RockboxStreams } from "@/lib/rockbox-streams";

SplashScreen.preventAutoHideAsync().catch(() => {});

const navTheme = {
  dark: true,
  colors: {
    primary: Colors.accent,
    background: Colors.appBg,
    card: Colors.appBg,
    text: Colors.textPrimary,
    border: Colors.divider,
    notification: Colors.accent,
  },
  fonts: {
    regular: { fontFamily: "SpaceGrotesk", fontWeight: "400" as const },
    medium: { fontFamily: "SpaceGrotesk", fontWeight: "500" as const },
    bold: { fontFamily: "SpaceGrotesk", fontWeight: "700" as const },
    heavy: { fontFamily: "SpaceGrotesk", fontWeight: "800" as const },
  },
};

export const unstable_settings = {
  anchor: "(tabs)",
};

export default function RootLayout() {
  const [fontsLoaded] = useFonts({
    SpaceGrotesk: require("@/assets/fonts/SpaceGrotesk.ttf"),
    JetBrainsMono: require("@/assets/fonts/JetBrainsMono.ttf"),
    "RockfordSans-Light": require("@/assets/fonts/RockfordSans-Light.otf"),
    "RockfordSans-Regular": require("@/assets/fonts/RockfordSans-Regular.otf"),
    "RockfordSans-RegularItalic": require("@/assets/fonts/RockfordSans-RegularItalic.otf"),
    "RockfordSans-Medium": require("@/assets/fonts/RockfordSans-Medium.otf"),
    "RockfordSans-Bold": require("@/assets/fonts/RockfordSans-Bold.otf"),
    "RockfordSans-BoldItalic": require("@/assets/fonts/RockfordSans-BoldItalic.otf"),
    "RockfordSans-ExtraBold": require("@/assets/fonts/RockfordSans-ExtraBold.otf"),
  });
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5 * 60 * 1000,
            // Retry up to 3× with exponential backoff (1s, 2s, 4s) so a
            // brief gRPC hiccup on JS reload or app foreground doesn't leave
            // screens permanently empty.
            retry: 3,
            retryDelay: (attempt) => Math.min(1000 * 2 ** attempt, 30_000),
          },
        },
      }),
  );

  useEffect(() => {
    if (fontsLoaded) SplashScreen.hideAsync().catch(() => {});
  }, [fontsLoaded]);

  // Android API 33+: ask the user once for "All files access" so the
  // embedded daemon's filesystem-based scanner can read /storage/emulated/0/Music.
  // No-op on iOS / web (hasAllFilesAccess() always returns true there).
  // Re-checks when the app foregrounds so the prompt clears once granted.
  useAllFilesAccessPrompt();

  if (!fontsLoaded) return null;

  return (
    <ThemeProvider value={navTheme}>
      <QueryClientProvider client={queryClient}>
      <RockboxStreams />
      <PlayerProvider>
        <Stack
          screenOptions={{
            headerShown: false,
            contentStyle: { backgroundColor: Colors.appBg },
          }}
        >
          <Stack.Screen name="(tabs)" />
          <Stack.Screen
            name="player"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="queue"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="settings"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="settings/server"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="settings/bluetooth"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="album/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="artist/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="playlist/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="playlist/new"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="genre/[id]"
            options={{ animation: "slide_from_right" }}
          />
        </Stack>
        <PersistentMiniPlayer />
        <TrackContextMenu />
        <StatusBar style="light" />
      </PlayerProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
}

function useAllFilesAccessPrompt() {
  const askedThisSession = useRef(false);

  useEffect(() => {
    if (Platform.OS !== "android") return;

    const check = () => {
      if (askedThisSession.current) return;
      // The native check returns true on iOS/web and on pre-API-30 Android,
      // so we'll only ever prompt on Android 11+ where it's actually needed.
      let granted = true;
      try {
        granted = RockboxClient.hasAllFilesAccess();
      } catch {
        return;
      }
      if (granted) return;

      askedThisSession.current = true;
      Alert.alert(
        "Allow access to your music",
        "Rockbox needs \"All files access\" to scan your Music folder. " +
          "Tap Open Settings, toggle the switch on, then come back.",
        [
          { text: "Not now", style: "cancel" },
          {
            text: "Open Settings",
            onPress: () => {
              try {
                RockboxClient.requestAllFilesAccess();
              } catch {
                // Settings intent failure — silently ignore; the user can
                // still grant from Apps → Rockbox manually.
              }
            },
          },
        ],
        { cancelable: true },
      );
    };

    check();
    const sub = AppState.addEventListener("change", (state) => {
      if (state === "active") {
        // Reset the once-per-session lock when the user comes back from
        // Settings so a "no" then "yes" round-trip clears the alert.
        askedThisSession.current = false;
        check();
      }
    });
    return () => sub.remove();
  }, []);
}
