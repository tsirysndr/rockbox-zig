import { Ionicons } from "@expo/vector-icons";
import { router, Stack } from "expo-router";
import { useState } from "react";
import { Pressable, ScrollView, Switch, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { Colors } from "@/constants/theme";

type Section = {
  title: string;
  rows: Row[];
};

type Row =
  | {
      kind: "switch";
      label: string;
      icon: React.ComponentProps<typeof Ionicons>["name"];
      value: boolean;
      onChange: (v: boolean) => void;
    }
  | {
      kind: "link";
      label: string;
      icon: React.ComponentProps<typeof Ionicons>["name"];
      value?: string;
      onPress?: () => void;
    };

export default function SettingsScreen() {
  const [crossfade, setCrossfade] = useState(true);
  const [normalize, setNormalize] = useState(true);
  const [downloadOnWifi, setDownloadOnWifi] = useState(true);
  const [hapticFeedback, setHapticFeedback] = useState(true);

  const sections: Section[] = [
    {
      title: "Account",
      rows: [
        {
          kind: "link",
          label: "Profile",
          icon: "person-outline",
          value: "tsiry.sndr@gmail.com",
        },
        {
          kind: "link",
          label: "Subscription",
          icon: "card-outline",
          value: "Free",
        },
        { kind: "link", label: "Sign out", icon: "log-out-outline" },
      ],
    },
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
        {
          kind: "link",
          label: "Audio quality",
          icon: "musical-notes-outline",
          value: "High",
        },
        { kind: "link", label: "Equalizer", icon: "options-outline" },
      ],
    },
    {
      title: "Storage",
      rows: [
        {
          kind: "switch",
          label: "Download over Wi-Fi only",
          icon: "wifi-outline",
          value: downloadOnWifi,
          onChange: setDownloadOnWifi,
        },
        {
          kind: "link",
          label: "Download quality",
          icon: "cloud-download-outline",
          value: "Very High",
        },
        {
          kind: "link",
          label: "Storage location",
          icon: "folder-outline",
          value: "Internal",
        },
      ],
    },
    {
      title: "Devices",
      rows: [
        { kind: "link", label: "Connect a device", icon: "bluetooth-outline" },
        { kind: "link", label: "AirPlay & Cast", icon: "tv-outline" },
        {
          kind: "link",
          label: "Rockbox server",
          icon: "server-outline",
          value: "localhost:6061",
        },
      ],
    },
    {
      title: "App",
      rows: [
        {
          kind: "switch",
          label: "Haptic feedback",
          icon: "phone-portrait-outline",
          value: hapticFeedback,
          onChange: setHapticFeedback,
        },
        {
          kind: "link",
          label: "Language",
          icon: "language-outline",
          value: "English",
        },
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
          <Text className="text-text-primary text-[22px] font-extrabold font-sans">
            Settings
          </Text>
        </View>

        <ScrollView
          contentContainerStyle={{ paddingBottom: 32 }}
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
    </>
  );
}

function SettingsRow({ row }: { row: Row }) {
  const content = (
    <View className="flex-row items-center px-3.5 h-[52px] gap-3.5">
      <Ionicons name={row.icon} size={20} color={Colors.textSecondary} />
      <Text className="flex-1 text-text-primary text-[15px] font-sans">
        {row.label}
      </Text>
      {row.kind === "switch" ? (
        <Switch
          value={row.value}
          onValueChange={row.onChange}
          trackColor={{ false: Colors.bgHover, true: Colors.accent }}
          thumbColor="#FFFFFF"
          ios_backgroundColor={Colors.bgHover}
        />
      ) : (
        <>
          {row.value ? (
            <Text className="text-text-secondary text-[13px] font-sans">
              {row.value}
            </Text>
          ) : null}
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
