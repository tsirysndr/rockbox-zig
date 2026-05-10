import { Ionicons } from "@expo/vector-icons";
import { IconDeviceLaptop } from "@tabler/icons-react-native";
import { router } from "expo-router";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  KeyboardAvoidingView,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { Colors } from "@/constants/theme";
import { useDiscoveredServers } from "@/lib/queries";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { RockboxClient, type DiscoveredService } from "@/lib/rockbox-client";
import { restartDiscovery } from "@/lib/rockbox-streams";
import {
  isLocalDiscovery,
  manualServer,
  serverFromDiscovery,
  setSelectedServer,
  useSelectedServer,
} from "@/lib/server-store";

export default function ServerPickerScreen() {
  const selected = useSelectedServer();
  const { data: discovered = [] } = useDiscoveredServers();
  const [host, setHost] = useState("");
  const [port, setPort] = useState("6061");
  const [scanning, setScanning] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  // Visible only when the native module is up — otherwise the discovery
  // stream and gRPC are no-ops and we just show the manual entry.
  const isAvailable = RockboxClient.isAvailable;
  const bottomPad = useBottomSpacing(24);

  // Show every host the discovery stream has resolved, regardless of which
  // flavor (grpc / graphql / http / mpd) advertised first. The mdns-sd
  // browser sometimes loses a flavor's `ServiceResolved` to a transient
  // network blip; if we filtered to grpc-only the user would just see an
  // empty list. Dedupe by host:port so we still get one row per machine.
  const grpcServices = useMemo(
    () => dedupeByHost(discovered).filter((svc) => !isLocalDiscovery(svc)),
    [discovered],
  );

  // Soft "scanning" indicator — true for ~3 s after entering the screen, then
  // hides so users see a stable list. Discovery itself runs continuously.
  useEffect(() => {
    if (!isAvailable) return;
    setScanning(true);
    const t = setTimeout(() => setScanning(false), 3000);
    return () => clearTimeout(t);
  }, [isAvailable]);

  const onPickDiscovered = async (svc: DiscoveredService) => {
    const sel = serverFromDiscovery(svc);
    if (!sel) return;
    await setSelectedServer(sel);
  };

  const onConnectManual = async () => {
    const trimmed = host.trim();
    if (!trimmed) return;
    const p = parseInt(port, 10);
    await setSelectedServer(manualServer(trimmed, isFinite(p) ? p : 6061));
    setHost("");
  };

  const onClear = async () => {
    await setSelectedServer(null);
  };

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    setScanning(true);
    // Tear down the active mdns-sd browse and start a fresh one — necessary
    // because mdns-sd only emits `ServiceResolved` the first time a service
    // is seen, so just clearing the cache would leave us with no events.
    restartDiscovery();
    // Keep the pull-to-refresh spinner alive briefly so a fast scan still
    // feels like a refresh, and the "scanning…" hint lingers longer.
    setTimeout(() => setRefreshing(false), 1200);
    setTimeout(() => setScanning(false), 2500);
  }, []);

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <View className="flex-row items-center px-4 py-3 gap-3 border-b border-divider">
        <Pressable hitSlop={10} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={26} color={Colors.textPrimary} />
        </Pressable>
        <Text className="text-text-primary text-[18px] font-display-extra">
          Rockbox server
        </Text>
      </View>

      <KeyboardAvoidingView
        className="flex-1"
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <ScrollView
          contentContainerStyle={{ paddingBottom: bottomPad }}
          keyboardShouldPersistTaps="handled"
          refreshControl={
            isAvailable ? (
              <RefreshControl
                refreshing={refreshing}
                onRefresh={onRefresh}
                tintColor={Colors.accent}
                colors={[Colors.accent]}
                progressBackgroundColor={Colors.bgCard}
              />
            ) : undefined
          }
        >
          {/* Current selection */}
          <View className="mt-4 px-4">
            <Text className="text-text-secondary text-xs font-bold uppercase tracking-widest mb-2 font-sans">
              Connected to
            </Text>
            <View className="bg-bg-card rounded-xl px-4 py-4">
              {selected ? (
                <View>
                  <Text className="text-text-primary text-[15px] font-display">
                    {selected.label}
                  </Text>
                  <Text className="text-text-secondary text-[13px] mt-1 font-mono">
                    {selected.host}:{selected.grpcPort}
                  </Text>
                  <Pressable
                    hitSlop={6}
                    onPress={onClear}
                    className="mt-3 self-start active:opacity-70"
                  >
                    <Text className="text-danger text-xs font-semibold font-sans">
                      Disconnect
                    </Text>
                  </Pressable>
                </View>
              ) : (
                <Text className="text-text-secondary text-[13px] font-sans">
                  No server selected.
                </Text>
              )}
            </View>
          </View>

          {/* Discovered servers */}
          <View className="mt-6 px-4">
            <View className="flex-row items-center justify-between mb-2">
              <Text className="text-text-secondary text-xs font-bold uppercase tracking-widest font-sans">
                Discovered on the network
              </Text>
              {scanning && isAvailable ? (
                <Text className="text-text-muted text-[11px] font-sans">
                  scanning…
                </Text>
              ) : null}
            </View>

            <View className="bg-bg-card rounded-xl overflow-hidden">
              {grpcServices.length === 0 ? (
                <View className="px-4 py-5">
                  <Text className="text-text-secondary text-[13px] font-sans">
                    {isAvailable
                      ? "No rockbox servers found yet. Make sure the daemon is running on the same network."
                      : "Native module not loaded — discovery is unavailable on this build."}
                  </Text>
                </View>
              ) : (
                grpcServices.map((svc, idx) => {
                  const sel = serverFromDiscovery(svc);
                  const isCurrent =
                    selected !== null &&
                    sel !== null &&
                    selected.host === sel.host &&
                    selected.grpcPort === sel.grpcPort;
                  return (
                    <View key={svc.fullname}>
                      {idx > 0 ? (
                        <View className="h-px bg-divider ml-12" />
                      ) : null}
                      <Pressable
                        onPress={() => onPickDiscovered(svc)}
                        android_ripple={{ color: Colors.bgHover }}
                        className="flex-row items-center px-4 py-3.5 gap-3 active:bg-bg-hover"
                      >
                        <View className="w-9 h-9 rounded-full bg-bg-elevated items-center justify-center">
                          <IconDeviceLaptop
                            size={20}
                            color={
                              isCurrent ? Colors.accent : Colors.textPrimary
                            }
                            strokeWidth={1.75}
                          />
                        </View>
                        <View className="flex-1">
                          <Text className="text-text-primary text-[14px] font-semibold font-sans">
                            {sel?.label ?? svc.hostname}
                          </Text>
                          <Text className="text-text-secondary text-[12px] mt-0.5 font-mono">
                            {sel?.host}:{sel?.grpcPort}
                          </Text>
                        </View>
                        {isCurrent ? (
                          <Ionicons
                            name="checkmark"
                            size={20}
                            color={Colors.accent}
                          />
                        ) : null}
                      </Pressable>
                    </View>
                  );
                })
              )}
            </View>
          </View>

          {/* Manual entry */}
          <View className="mt-6 px-4">
            <Text className="text-text-secondary text-xs font-bold uppercase tracking-widest mb-2 font-sans">
              Connect manually
            </Text>
            <View className="bg-bg-card rounded-xl px-4 py-3 gap-2.5">
              <TextInput
                value={host}
                onChangeText={setHost}
                placeholder="Hostname or IP"
                placeholderTextColor={Colors.textMuted}
                autoCapitalize="none"
                autoCorrect={false}
                className="text-text-primary text-[14px] font-mono"
              />
              <View className="h-px bg-divider" />
              <View className="flex-row items-center gap-2">
                <Text className="text-text-secondary text-[13px] font-sans">
                  Port
                </Text>
                <TextInput
                  value={port}
                  onChangeText={setPort}
                  keyboardType="number-pad"
                  className="flex-1 text-text-primary text-[14px] font-mono"
                />
              </View>
              <Pressable
                disabled={host.trim().length === 0}
                onPress={onConnectManual}
                className={`mt-2 self-start px-4 py-2 rounded-full ${host.trim().length === 0 ? "bg-bg-hover" : "bg-accent active:opacity-80"}`}
              >
                <Text className="text-white text-[13px] font-display">
                  Connect
                </Text>
              </Pressable>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

function dedupeByHost(list: DiscoveredService[]): DiscoveredService[] {
  const seen = new Set<string>();
  const out: DiscoveredService[] = [];
  for (const s of list) {
    const sel = serverFromDiscovery(s);
    if (!sel) continue;
    const key = `${sel.host}:${sel.grpcPort}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(s);
  }
  return out;
}
