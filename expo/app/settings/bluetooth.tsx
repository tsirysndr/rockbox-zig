import { Ionicons } from "@expo/vector-icons";
import { router } from "expo-router";
import { useCallback, useEffect, useState } from "react";
import { Pressable, RefreshControl, ScrollView, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { Colors } from "@/constants/theme";
import { useIsConnected } from "@/lib/connection";
import {
  useBluetoothAvailable,
  useBluetoothDevices,
  useConnectBluetooth,
  useDisconnectBluetooth,
  useScanBluetooth,
} from "@/lib/queries";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

type ProtoBluetoothDevice = {
  address?: string;
  name?: string;
  paired?: boolean;
  trusted?: boolean;
  connected?: boolean;
  rssi?: number | null;
};

export default function BluetoothScreen() {
  const isConnected = useIsConnected();
  const available = useBluetoothAvailable({ enabled: isConnected });
  const list = useBluetoothDevices<{ devices?: ProtoBluetoothDevice[] }>({
    enabled: isConnected && available.data === true,
  });
  const connectMut = useConnectBluetooth();
  const disconnectMut = useDisconnectBluetooth();
  const scanMut = useScanBluetooth();
  const [scanned, setScanned] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const bottomPad = useBottomSpacing(24);

  const onRefresh = useCallback(async () => {
    if (!isConnected || available.data !== true) return;
    setRefreshing(true);
    try {
      await scanMut.mutateAsync();
      await list.refetch();
    } catch {
      // swallow — UI stays on whatever the last good state was
    } finally {
      setRefreshing(false);
    }
  }, [isConnected, available.data, scanMut, list]);

  const devices = list.data?.devices ?? [];

  // Trigger one scan on mount once we know Bluetooth is available — rockboxd
  // doesn't continuously scan, so without this `getDevices` returns nothing.
  useEffect(() => {
    if (!isConnected || available.data !== true || scanned) return;
    setScanned(true);
    scanMut.mutate();
  }, [isConnected, available.data, scanned, scanMut]);

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <View className="flex-row items-center px-4 py-3 gap-3 border-b border-divider">
        <Pressable hitSlop={10} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={26} color={Colors.textPrimary} />
        </Pressable>
        <Text className="flex-1 text-text-primary text-[18px] font-display-extra">
          Bluetooth
        </Text>
        {available.data === true && isConnected ? (
          <Pressable
            hitSlop={10}
            onPress={() => scanMut.mutate()}
            disabled={scanMut.isPending}
            className={`px-3 h-8 rounded-full items-center justify-center flex-row gap-1.5 ${scanMut.isPending ? "bg-bg-card" : "bg-bg-elevated active:bg-bg-hover"}`}
          >
            <Ionicons
              name={scanMut.isPending ? "sync" : "search"}
              size={14}
              color={Colors.textPrimary}
            />
            <Text className="text-text-primary text-xs font-display">
              {scanMut.isPending ? "Scanning…" : "Scan"}
            </Text>
          </Pressable>
        ) : null}
      </View>

      <ScrollView
        contentContainerStyle={{ paddingBottom: bottomPad }}
        refreshControl={
          isConnected && available.data === true ? (
            <RefreshControl
              refreshing={refreshing || scanMut.isPending}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
              progressBackgroundColor={Colors.bgCard}
            />
          ) : undefined
        }
      >
        {!isConnected ? (
          <Banner
            icon="server-outline"
            text="Connect to a rockbox server first to manage Bluetooth devices."
          />
        ) : available.isLoading ? (
          <Banner icon="bluetooth-outline" text="Checking Bluetooth…" />
        ) : available.data === false ? (
          <Banner
            icon="alert-circle-outline"
            text="Bluetooth isn't available on this server."
          />
        ) : null}

        {isConnected && available.data === true ? (
          <View className="mt-4 px-4">
            <Text className="text-text-secondary text-xs font-bold uppercase tracking-widest mb-2 font-sans">
              Devices
            </Text>
            <View className="bg-bg-card rounded-xl overflow-hidden">
              {list.isLoading && devices.length === 0 ? (
                <Banner icon="bluetooth-outline" text="Scanning…" />
              ) : devices.length === 0 ? (
                <Banner
                  icon="bluetooth-outline"
                  text="No Bluetooth devices found."
                />
              ) : (
                devices.map((d, idx) => {
                  const isOn = d.connected === true;
                  const onToggle = () => {
                    if (!d.address) return;
                    if (isOn) disconnectMut.mutate(d.address);
                    else connectMut.mutate(d.address);
                  };
                  return (
                    <View key={d.address ?? idx}>
                      {idx > 0 ? (
                        <View className="h-px bg-divider ml-12" />
                      ) : null}
                      <Pressable
                        onPress={onToggle}
                        android_ripple={{ color: Colors.bgHover }}
                        className="flex-row items-center px-4 py-3.5 gap-3 active:bg-bg-hover"
                      >
                        <View
                          className={`w-9 h-9 rounded-full items-center justify-center ${isOn ? "bg-accent" : "bg-bg-elevated"}`}
                        >
                          <Ionicons
                            name="bluetooth"
                            size={18}
                            color={isOn ? "#FFFFFF" : Colors.textPrimary}
                          />
                        </View>
                        <View className="flex-1 min-w-0">
                          <Text
                            numberOfLines={1}
                            className="text-text-primary text-[14px] font-semibold font-sans"
                          >
                            {d.name?.trim() || "(unnamed device)"}
                          </Text>
                          <Text
                            numberOfLines={1}
                            className="text-text-secondary text-[12px] mt-0.5 font-mono"
                          >
                            {d.address ?? "—"}
                            {d.paired ? "  •  paired" : ""}
                            {typeof d.rssi === "number"
                              ? `  •  ${d.rssi} dBm`
                              : ""}
                          </Text>
                        </View>
                        <Text
                          className={`text-xs font-bold uppercase font-sans ${isOn ? "text-accent" : "text-text-muted"}`}
                        >
                          {isOn ? "Connected" : "Connect"}
                        </Text>
                      </Pressable>
                    </View>
                  );
                })
              )}
            </View>
          </View>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  );
}

function Banner({
  icon,
  text,
}: {
  icon: React.ComponentProps<typeof Ionicons>["name"];
  text: string;
}) {
  return (
    <View className="mt-4 mx-4 px-4 py-4 bg-bg-card rounded-xl flex-row items-center gap-3">
      <Ionicons name={icon} size={20} color={Colors.textSecondary} />
      <Text className="flex-1 text-text-secondary text-[13px] font-sans">
        {text}
      </Text>
    </View>
  );
}
