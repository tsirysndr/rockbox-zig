import { Ionicons } from "@expo/vector-icons";
import { router, useLocalSearchParams } from "expo-router";
import { useState } from "react";
import {
  KeyboardAvoidingView,
  Platform,
  Pressable,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { Colors } from "@/constants/theme";
import { usePlayer } from "@/lib/player-context";

const RULE_PRESETS = [
  'genre = "Indie"',
  "year >= 2020",
  "play_count > 5",
  "liked = true",
  "duration < 240",
  "added_within 30d",
];

export default function NewPlaylistScreen() {
  const { mode } = useLocalSearchParams<{ mode?: string }>();
  const isSmart = mode === "smart";
  const { createPlaylist } = usePlayer();

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [rules, setRules] = useState("");

  const canCreate = name.trim().length > 0;

  const onCreate = () => {
    if (!canCreate) return;
    const created = createPlaylist({
      name,
      description,
      isSmart,
      rules: isSmart ? rules : undefined,
    });
    router.back();
    setTimeout(() => router.push(`/playlist/${created.id}`), 50);
  };

  const insertRule = (snippet: string) =>
    setRules((current) => (current ? `${current}\n${snippet}` : snippet));

  return (
    <SafeAreaView className="flex-1 bg-bg">
      <KeyboardAvoidingView
        className="flex-1"
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <View className="flex-row items-center justify-between px-4 py-3 border-b border-divider">
          <Pressable hitSlop={10} onPress={() => router.back()}>
            <Text className="text-text-primary text-sm font-sans">Cancel</Text>
          </Pressable>
          <Text className="text-text-primary text-base font-bold font-sans">
            {isSmart ? "New Smart Playlist" : "New Playlist"}
          </Text>
          <Pressable hitSlop={10} disabled={!canCreate} onPress={onCreate}>
            <Text
              className={`text-sm font-bold font-sans ${canCreate ? "text-accent" : "text-text-muted"}`}
            >
              Create
            </Text>
          </Pressable>
        </View>

        <ScrollView
          contentContainerStyle={{ paddingBottom: 32 }}
          keyboardShouldPersistTaps="handled"
        >
          <View className="items-center py-6">
            <View
              className="w-[140px] h-[140px] rounded-lg bg-bg-card items-center justify-center"
              style={{
                shadowColor: "#000",
                shadowOpacity: 0.5,
                shadowRadius: 16,
                shadowOffset: { width: 0, height: 8 },
              }}
            >
              <Ionicons
                name={isSmart ? "flash" : "musical-notes"}
                size={56}
                color={Colors.textMuted}
              />
            </View>
          </View>

          <Section label="Name">
            <TextInput
              value={name}
              onChangeText={setName}
              placeholder={isSmart ? "Smart playlist name" : "My playlist"}
              placeholderTextColor={Colors.textMuted}
              autoFocus
              maxLength={64}
              className="bg-bg-card rounded-md text-text-primary text-[15px] px-3.5 py-3 font-sans"
            />
          </Section>

          <Section label="Description">
            <TextInput
              value={description}
              onChangeText={setDescription}
              placeholder="Optional description"
              placeholderTextColor={Colors.textMuted}
              multiline
              numberOfLines={3}
              maxLength={240}
              textAlignVertical="top"
              className="bg-bg-card rounded-md text-text-primary text-[15px] px-3.5 pt-3 pb-3 min-h-[76px] font-sans"
            />
          </Section>

          {isSmart ? (
            <>
              <Section label="Rules">
                <TextInput
                  value={rules}
                  onChangeText={setRules}
                  placeholder={`one rule per line, e.g.\ngenre = "Indie"\nyear >= 2020`}
                  placeholderTextColor={Colors.textMuted}
                  multiline
                  numberOfLines={6}
                  textAlignVertical="top"
                  className="bg-bg-card rounded-md text-text-primary text-[13px] px-3.5 pt-3 pb-3 min-h-[140px] font-mono"
                />
              </Section>

              <View className="px-4 mt-1">
                <Text className="text-text-secondary text-xs font-bold mb-2 uppercase tracking-widest font-sans">
                  Suggestions
                </Text>
                <View className="flex-row flex-wrap gap-2">
                  {RULE_PRESETS.map((preset) => (
                    <Pressable
                      key={preset}
                      onPress={() => insertRule(preset)}
                      className="px-3 h-[30px] rounded-full bg-bg-card items-center justify-center flex-row gap-1.5 active:opacity-70"
                    >
                      <Ionicons
                        name="add"
                        size={14}
                        color={Colors.textSecondary}
                      />
                      <Text className="text-text-secondary text-xs font-mono">
                        {preset}
                      </Text>
                    </Pressable>
                  ))}
                </View>
              </View>
            </>
          ) : null}
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

function Section({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <View className="px-4 mb-4">
      <Text className="text-text-secondary text-xs font-bold mb-2 uppercase tracking-widest font-sans">
        {label}
      </Text>
      {children}
    </View>
  );
}
