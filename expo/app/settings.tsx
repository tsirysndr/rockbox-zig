import { Ionicons } from "@expo/vector-icons";
import { IconCast, type Icon as TablerIcon } from "@tabler/icons-react-native";
import { router, Stack } from "expo-router";
import { useState } from "react";
import { Pressable, ScrollView, Switch, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import {
  DevicePickerSheet,
  useCurrentDeviceLabel,
} from "@/components/device-picker";
import { Colors } from "@/constants/theme";
import { useSelectedServer } from "@/lib/server-store";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

type Section = {
  title: string;
  rows: Row[];
};

type IconName = React.ComponentProps<typeof Ionicons>["name"];

type Row =
  | {
      kind: "switch";
      label: string;
      icon: IconName;
      tablerIcon?: TablerIcon;
      value: boolean;
      onChange: (v: boolean) => void;
    }
  | {
      kind: "link";
      label: string;
      icon: IconName;
      tablerIcon?: TablerIcon;
      value?: string;
      onPress?: () => void;
    };

export default function SettingsScreen() {
  const [crossfade, setCrossfade] = useState(true);
  const [normalize, setNormalize] = useState(true);
  const selectedServer = useSelectedServer();
  const serverLabel = selectedServer
    ? `${selectedServer.label} (${selectedServer.host}:${selectedServer.grpcPort})`
    : "Not connected";
  const [devicePickerOpen, setDevicePickerOpen] = useState(false);
  const currentDevice = useCurrentDeviceLabel();
  const bottomPad = useBottomSpacing(24);

  const sections: Section[] = [
    {
      title: "Playback",
      rows: [
        {
          kind: "switch",
          label: "Crossfade",
          icon: "swap-horizontal-outline",
          value: crossfade,
          onChange: setCrossfade,
        },
        {
          kind: "switch",
          label: "Normalize volume",
          icon: "stats-chart-outline",
          value: normalize,
          onChange: setNormalize,
        },
        { kind: "link", label: "Equalizer", icon: "options-outline" },
      ],
    },
    {
      title: "Devices",
      rows: [
        {
          kind: "link",
          label: "Bluetooth",
          icon: "bluetooth-outline",
          onPress: () => router.push("/settings/bluetooth"),
        },
        {
          kind: "link",
          label: "AirPlay & Cast",
          icon: "tv-outline",
          tablerIcon: IconCast,
          value: currentDevice.name,
          onPress: () => setDevicePickerOpen(true),
        },
        {
          kind: "link",
          label: "Rockbox server",
          icon: "server-outline",
          value: serverLabel,
          onPress: () => router.push("/settings/server"),
        },
      ],
    },
    {
      title: "App",
      rows: [
        {
          kind: "link",
          label: "About",
          icon: "information-circle-outline",
          value: "1.0.0",
        },
      ],
    },
  ];

  return (
    <>
      <Stack.Screen options={{ headerShown: false }} />
      <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
        <View className="flex-row items-center px-4 py-3 gap-3">
          <Pressable hitSlop={10} onPress={() => router.back()}>
            <Ionicons
              name="chevron-back"
              size={26}
              color={Colors.textPrimary}
            />
          </Pressable>
          <Text className="text-text-primary text-[22px] font-display-extra">
            Settings
          </Text>
        </View>

        <ScrollView
          contentContainerStyle={{ paddingBottom: bottomPad }}
          showsVerticalScrollIndicator={false}
        >
          {sections.map((section) => (
            <View key={section.title} className="mb-6">
              <Text className="text-text-secondary text-xs font-bold tracking-widest px-4 pb-2 uppercase font-sans">
                {section.title}
              </Text>
              <View className="mx-4 rounded-xl bg-bg-card overflow-hidden">
                {section.rows.map((row, idx) => (
                  <View key={`${section.title}-${row.label}`}>
                    {idx > 0 ? (
                      <View className="h-px bg-divider ml-[52px]" />
                    ) : null}
                    <SettingsRow row={row} />
                  </View>
                ))}
              </View>
            </View>
          ))}
        </ScrollView>
      </SafeAreaView>
      <DevicePickerSheet
        visible={devicePickerOpen}
        onClose={() => setDevicePickerOpen(false)}
      />
    </>
  );
}

function SettingsRow({ row }: { row: Row }) {
  const TablerIconComp = row.tablerIcon;
  const content = (
    <View className="flex-row items-center px-3.5 h-[52px] gap-3.5">
      {TablerIconComp ? (
        <TablerIconComp
          size={20}
          color={Colors.textSecondary}
          strokeWidth={1.75}
        />
      ) : (
        <Ionicons name={row.icon} size={20} color={Colors.textSecondary} />
      )}
      <Text
        numberOfLines={1}
        className="text-text-primary text-[15px] font-sans flex-shrink-0"
      >
        {row.label}
      </Text>
      {row.kind === "switch" ? (
        <>
          <View className="flex-1" />
          <Switch
            value={row.value}
            onValueChange={row.onChange}
            trackColor={{ false: Colors.bgHover, true: Colors.accent }}
            thumbColor="#FFFFFF"
            ios_backgroundColor={Colors.bgHover}
          />
        </>
      ) : (
        <>
          {row.value ? (
            <Text
              numberOfLines={1}
              ellipsizeMode="tail"
              className="flex-1 min-w-0 text-right text-text-secondary text-[13px] font-sans"
            >
              {row.value}
            </Text>
          ) : (
            <View className="flex-1" />
          )}
          <Ionicons
            name="chevron-forward"
            size={18}
            color={Colors.textMuted}
          />
        </>
      )}
    </View>
  );

  if (row.kind === "link") {
    return (
      <Pressable
        onPress={row.onPress}
        android_ripple={{ color: Colors.bgHover }}
        className="active:bg-bg-hover"
      >
        {content}
      </Pressable>
    );
  }
  return content;
}
