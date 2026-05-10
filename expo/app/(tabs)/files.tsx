import { Ionicons } from "@expo/vector-icons";
import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import {
  ActivityIndicator,
  Alert,
  BackHandler,
  FlatList,
  Modal,
  Pressable,
  RefreshControl,
  Text,
  View,
} from "react-native";
import { useFocusEffect } from "expo-router";
import { SafeAreaView } from "react-native-safe-area-context";

import { NotConnectedState } from "@/components/empty-state";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { useIsConnected } from "@/lib/connection";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { usePlayer } from "@/lib/player-context";
import { RockboxClient } from "@/lib/rockbox-client";
import type { FileEntry, FilesMode } from "@/lib/types";

// ── Insert position constants (matches apps/playlist.h) ───────────────────────

const INSERT_NEXT = -4;           // INSERT_FIRST: after current track
const INSERT_LAST = -3;           // INSERT_LAST: append to end
const INSERT_SHUFFLED = -5;       // INSERT_SHUFFLED: insert shuffled next
const INSERT_LAST_SHUFFLED = -7;  // INSERT_LAST_SHUFFLED: append shuffled

// ── Entry parsing ─────────────────────────────────────────────────────────────

type RawEntry = {
  name: string;
  attr: number;
  displayName?: string | null;
  display_name?: string | null;
};

function toFileEntry(e: RawEntry): FileEntry {
  const displayName = e.displayName ?? e.display_name ?? null;
  const parts = e.name.split("/").filter(Boolean);
  const basename = parts[parts.length - 1] ?? e.name;
  return {
    name: displayName || basename,
    path: e.name,
    is_dir: e.attr === 0x10,
  };
}

function sortEntries(arr: FileEntry[]): FileEntry[] {
  return [...arr].sort((a, b) => {
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
    return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
  });
}

// Infer the parent directory path from a list of entries.
function inferCurrentDir(entries: FileEntry[]): string | null {
  const first = entries[0];
  if (!first) return null;
  const parts = first.path.split("/").filter(Boolean);
  parts.pop();
  return parts.length ? "/" + parts.join("/") : null;
}

// ── Browse state ──────────────────────────────────────────────────────────────

type BrowseState = {
  mode: FilesMode;
  path: string | null;
  // Display name of the current folder/device — set from entry.name on navigate
  displayLabel: string | null;
  history: Array<{ mode: FilesMode; path: string | null; displayLabel: string | null }>;
};

type BrowseAction =
  | { type: "push"; mode: FilesMode; path: string | null; displayLabel?: string }
  | { type: "pop" };

function browseReducer(state: BrowseState, action: BrowseAction): BrowseState {
  if (action.type === "push") {
    return {
      mode: action.mode,
      path: action.path,
      displayLabel: action.displayLabel ?? null,
      history: [
        ...state.history,
        { mode: state.mode, path: state.path, displayLabel: state.displayLabel },
      ],
    };
  }
  if (action.type === "pop" && state.history.length > 0) {
    const prev = state.history[state.history.length - 1];
    return {
      mode: prev.mode,
      path: prev.path,
      displayLabel: prev.displayLabel,
      history: state.history.slice(0, -1),
    };
  }
  return state;
}

const INITIAL: BrowseState = { mode: "root", path: null, displayLabel: null, history: [] };

// ── Header label ──────────────────────────────────────────────────────────────

function headerLabel(
  mode: FilesMode,
  path: string | null,
  displayLabel: string | null,
): string {
  if (displayLabel) return displayLabel;
  if (mode === "root") return "Files";
  if (mode === "local") return "Music";
  if (mode === "upnp-devices") return "UPnP Devices";
  if (mode === "upnp-browse") return "UPnP";
  return "Files";
}

// ── Resolve fetch path ────────────────────────────────────────────────────────

function fetchPath(mode: FilesMode, path: string | null): string | null {
  if (mode === "local") return path; // null → music root on server
  if (mode === "upnp-devices") return "upnp://";
  if (mode === "upnp-browse") return path;
  return null;
}

// ── Context menu sheet ────────────────────────────────────────────────────────

type MenuEntry = {
  entry: FileEntry;
  currentDir: string | null;
  entryIndex: number;
};

function FileActionsSheet({
  menu,
  onClose,
}: {
  menu: MenuEntry | null;
  onClose: () => void;
}) {
  if (!menu) {
    return (
      <Modal visible={false} transparent animationType="slide" onRequestClose={onClose} />
    );
  }

  const { entry, currentDir, entryIndex } = menu;
  const isDir = entry.is_dir;
  const { queue } = usePlayer();

  const run = (action: () => Promise<unknown>) => {
    action().catch(() => {});
    onClose();
  };

  const confirmPlay = (action: () => void) => {
    if (queue.length > 0) {
      Alert.alert("Replace Queue", "This will clear the current queue.", [
        { text: "Cancel", style: "cancel" },
        { text: "Play", style: "destructive", onPress: action },
      ]);
    } else {
      action();
    }
    onClose();
  };

  type MenuItem = {
    icon: React.ComponentProps<typeof Ionicons>["name"];
    label: string;
    separator?: boolean;
    onPress: () => void;
  };

  const items: MenuItem[] = [
    {
      icon: "play-circle-outline",
      label: "Play",
      onPress: () =>
        confirmPlay(() =>
          (isDir
            ? RockboxClient.playDirectory(entry.path, false)
            : RockboxClient.playDirectory(currentDir ?? entry.path, false, entryIndex)
          ).catch(() => {}),
        ),
    },
    {
      icon: "play-skip-forward-outline",
      label: "Play Next",
      onPress: () =>
        run(() =>
          isDir
            ? RockboxClient.insertDirectory(entry.path, INSERT_NEXT)
            : RockboxClient.insertTrackNext(entry.path),
        ),
    },
    {
      icon: "list-outline",
      label: "Play Last",
      onPress: () =>
        run(() =>
          isDir
            ? RockboxClient.insertDirectory(entry.path, INSERT_LAST)
            : RockboxClient.insertTrackLast(entry.path),
        ),
    },
    {
      icon: "shuffle-outline",
      label: "Add Shuffled",
      onPress: () =>
        run(() =>
          isDir
            ? RockboxClient.insertDirectory(entry.path, INSERT_SHUFFLED)
            : RockboxClient.insertTracks([entry.path], INSERT_SHUFFLED, false),
        ),
    },
    // Directory-only extras
    ...(isDir
      ? ([
          {
            icon: "shuffle-outline",
            label: "Play Last Shuffled",
            separator: true,
            onPress: () =>
              run(() => RockboxClient.insertDirectory(entry.path, INSERT_LAST_SHUFFLED)),
          },
          {
            icon: "play-circle",
            label: "Play Shuffled",
            onPress: () =>
              confirmPlay(() =>
                RockboxClient.playDirectory(entry.path, true).catch(() => {}),
              ),
          },
        ] as MenuItem[])
      : []),
  ];

  return (
    <Modal
      visible
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <Pressable onPress={onClose} className="flex-1 bg-black/55">
        <Pressable
          onPress={(e) => e.stopPropagation()}
          className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
        >
          <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />

          {/* Header */}
          <View className="flex-row items-center gap-3 px-4 py-3 border-b border-divider">
            <View className="w-10 h-10 rounded-lg bg-bg-card items-center justify-center shrink-0">
              {isDir ? (
                <Ionicons name="folder-outline" size={22} color={Colors.textMuted} />
              ) : (
                <Ionicons name="musical-note" size={22} color={Colors.textMuted} />
              )}
            </View>
            <Text
              numberOfLines={2}
              className="flex-1 text-text-primary text-[15px] font-display-medium"
            >
              {entry.name}
            </Text>
          </View>

          {/* Actions */}
          {items.map((item) => (
            <View key={item.label}>
              {item.separator ? (
                <View className="h-px bg-divider mx-4 my-1" />
              ) : null}
              <Pressable
                onPress={item.onPress}
                android_ripple={{ color: Colors.bgHover }}
                className="flex-row items-center gap-4 px-5 py-3.5 active:bg-bg-hover"
              >
                <Ionicons name={item.icon} size={22} color={Colors.textPrimary} />
                <Text className="text-text-primary text-[15px] font-sans">
                  {item.label}
                </Text>
              </Pressable>
            </View>
          ))}
        </Pressable>
      </Pressable>
    </Modal>
  );
}

// ── Root tiles ────────────────────────────────────────────────────────────────

function RootTile({
  icon,
  label,
  onPress,
}: {
  icon: React.ReactNode;
  label: string;
  onPress: () => void;
}) {
  return (
    <Pressable
      onPress={onPress}
      className="flex-1 bg-bg-card rounded-xl p-5 gap-3 active:opacity-70"
    >
      {icon}
      <Text className="text-text-primary text-[15px] font-display-medium">
        {label}
      </Text>
    </Pressable>
  );
}

// ── File row ──────────────────────────────────────────────────────────────────

function FileRow({
  entry,
  index,
  currentDir,
  onNavigate,
  onPlay,
  onLongPress,
}: {
  entry: FileEntry;
  index: number;
  currentDir: string | null;
  onNavigate: (entry: FileEntry) => void;
  onPlay: (entry: FileEntry, index: number, dir: string | null) => void;
  onLongPress: (entry: FileEntry, index: number, dir: string | null) => void;
}) {
  return (
    <Pressable
      onPress={() =>
        entry.is_dir ? onNavigate(entry) : onPlay(entry, index, currentDir)
      }
      onLongPress={() => onLongPress(entry, index, currentDir)}
      className="flex-row items-center px-4 py-3 gap-3 active:bg-bg-card"
    >
      <View className="w-9 h-9 rounded-lg bg-bg-card items-center justify-center shrink-0">
        {entry.is_dir ? (
          <Ionicons name="folder-outline" size={20} color={Colors.textMuted} />
        ) : (
          <Ionicons name="musical-note" size={20} color={Colors.textMuted} />
        )}
      </View>
      <Text
        className="flex-1 text-text-primary text-[14px] font-sans"
        numberOfLines={1}
      >
        {entry.name}
      </Text>
      <Pressable
        hitSlop={8}
        onPress={() => onLongPress(entry, index, currentDir)}
      >
        <Ionicons
          name="ellipsis-horizontal"
          size={18}
          color={Colors.textMuted}
        />
      </Pressable>
    </Pressable>
  );
}

// ── Device row (UPnP device list) ─────────────────────────────────────────────

function DeviceRow({
  entry,
  onPress,
}: {
  entry: FileEntry;
  onPress: (entry: FileEntry) => void;
}) {
  return (
    <Pressable
      onPress={() => onPress(entry)}
      className="flex-row items-center px-4 py-3 gap-3 active:bg-bg-card"
    >
      <View className="w-9 h-9 rounded-lg bg-bg-card items-center justify-center shrink-0">
        <Ionicons name="laptop-outline" size={20} color={Colors.textMuted} />
      </View>
      <Text
        className="flex-1 text-text-primary text-[14px] font-sans"
        numberOfLines={1}
      >
        {entry.name}
      </Text>
    </Pressable>
  );
}

// ── Main screen ───────────────────────────────────────────────────────────────

export default function FilesScreen() {
  const [browse, dispatch] = useReducer(browseReducer, INITIAL);
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [menu, setMenu] = useState<MenuEntry | null>(null);
  const bottomPad = useBottomSpacing(16);
  const isConnected = useIsConnected();
  const { queue } = usePlayer();
  const fetchSeq = useRef(0);
  // Persistent cache keyed by "mode:path" — survives back/forward navigation
  const cache = useRef<Map<string, FileEntry[]>>(new Map());

  const canGoBack = browse.history.length > 0;
  const label = headerLabel(browse.mode, browse.path, browse.displayLabel);

  const doFetch = useCallback(
    (mode: FilesMode, path: string | null, isRefresh: boolean) => {
      if (!isConnected) return;
      const seq = ++fetchSeq.current;
      const key = `${mode}:${path ?? ""}`;
      const cached = cache.current.get(key);

      if (isRefresh) {
        setRefreshing(true);
      } else if (cached) {
        // Show stale data instantly; background-refresh without a spinner
        setEntries(cached);
      } else {
        setLoading(true);
      }

      RockboxClient.treeGetEntries(fetchPath(mode, path))
        .then((res: unknown) => {
          if (fetchSeq.current !== seq) return;
          const raw = (res as { entries?: RawEntry[] })?.entries ?? [];
          const sorted = sortEntries(raw.map(toFileEntry));
          cache.current.set(key, sorted);
          setEntries(sorted);
        })
        .catch(() => {
          if (fetchSeq.current !== seq) return;
          // Keep stale data on error; only clear if nothing was cached
          if (!cache.current.has(key)) setEntries([]);
        })
        .finally(() => {
          if (fetchSeq.current !== seq) return;
          setLoading(false);
          setRefreshing(false);
        });
    },
    [isConnected],
  );

  useEffect(() => {
    if (browse.mode === "root") {
      setEntries([]);
      return;
    }
    doFetch(browse.mode, browse.path, false);
  }, [browse.mode, browse.path, doFetch]);

  const handleRefresh = useCallback(() => {
    doFetch(browse.mode, browse.path, true);
  }, [browse.mode, browse.path, doFetch]);

  const navigate = useCallback(
    (mode: FilesMode, path: string | null, displayLabel?: string) =>
      dispatch({ type: "push", mode, path, displayLabel }),
    [],
  );

  const goBack = useCallback(() => dispatch({ type: "pop" }), []);

  // Intercept Android back gesture/button: go up the directory tree instead of
  // leaving the Files tab. Returns true (consumed) only when inside a directory.
  useFocusEffect(
    useCallback(() => {
      const sub = BackHandler.addEventListener("hardwareBackPress", () => {
        if (browse.history.length > 0) {
          dispatch({ type: "pop" });
          return true;
        }
        return false;
      });
      return () => sub.remove();
    }, [browse.history.length]),
  );

  const handleNavigate = useCallback(
    (entry: FileEntry) => {
      const nextMode: FilesMode =
        browse.mode === "upnp-devices" || browse.mode === "upnp-browse"
          ? "upnp-browse"
          : "local";
      navigate(nextMode, entry.path, entry.name);
    },
    [browse.mode, navigate],
  );

  const handlePlay = useCallback(
    (entry: FileEntry, index: number, dir: string | null) => {
      const currentDir = dir ?? inferCurrentDir(entries);
      const doPlay = () =>
        RockboxClient.playDirectory(currentDir ?? entry.path, false, index).catch(() => {});
      if (queue.length > 0) {
        Alert.alert("Replace Queue", "This will clear the current queue.", [
          { text: "Cancel", style: "cancel" },
          { text: "Play", style: "destructive", onPress: doPlay },
        ]);
      } else {
        doPlay();
      }
    },
    [entries, queue.length],
  );

  const openMenu = useCallback(
    (entry: FileEntry, index: number, dir: string | null) => {
      const currentDir = dir ?? inferCurrentDir(entries);
      setMenu({ entry, currentDir, entryIndex: index });
    },
    [entries],
  );

  const isUpnpMode =
    browse.mode === "upnp-devices" || browse.mode === "upnp-browse";

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <RemoteServerBanner />
      {/* Header */}
      <View className="flex-row items-center px-4 pt-2 pb-3 gap-3">
        {canGoBack ? (
          <Pressable
            onPress={goBack}
            hitSlop={10}
            className="w-8 h-8 rounded-full bg-bg-card items-center justify-center shrink-0 active:opacity-70"
          >
            <Ionicons name="chevron-back" size={20} color={Colors.textPrimary} />
          </Pressable>
        ) : null}
        <Text
          numberOfLines={1}
          className={`flex-1 text-text-primary font-display-extra ${canGoBack ? "text-[20px]" : "text-[26px]"}`}
        >
          {label}
        </Text>
      </View>

      {/* Root tiles */}
      {browse.mode === "root" ? (
        <View className="flex-row gap-3 px-4 pt-2">
          <RootTile
            icon={<Ionicons name="folder-open-outline" size={32} color={Colors.textMuted} />}
            label="Music"
            onPress={() => navigate("local", null)}
          />
          <RootTile
            icon={<Ionicons name="laptop-outline" size={32} color={Colors.textMuted} />}
            label="UPnP Devices"
            onPress={() => navigate("upnp-devices", null)}
          />
        </View>
      ) : null}

      {/* Non-root: connection guard */}
      {browse.mode !== "root" && !isConnected ? (
        <View className="flex-1">
          <NotConnectedState />
        </View>
      ) : null}

      {/* Full-screen spinner — only when there's no cached data to show */}
      {browse.mode !== "root" && isConnected && loading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator size="large" color={Colors.accent} />
        </View>
      ) : null}

      {/* File / device list */}
      {browse.mode !== "root" && isConnected && !loading ? (
        <FlatList
          data={entries}
          keyExtractor={(item) => item.path}
          contentContainerStyle={{ paddingBottom: bottomPad }}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={handleRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          renderItem={({ item, index }) =>
            isUpnpMode && browse.mode === "upnp-devices" ? (
              <DeviceRow
                entry={item}
                onPress={handleNavigate}
              />
            ) : (
              <FileRow
                entry={item}
                index={index}
                currentDir={browse.path}
                onNavigate={handleNavigate}
                onPlay={handlePlay}
                onLongPress={openMenu}
              />
            )
          }
          ListEmptyComponent={
            <View className="flex-1 items-center justify-center py-16">
              <Ionicons
                name="folder-open-outline"
                size={48}
                color={Colors.textMuted}
              />
              <Text className="text-text-muted text-[14px] mt-3">
                {browse.mode === "upnp-devices"
                  ? "No UPnP devices found"
                  : "No files here"}
              </Text>
            </View>
          }
        />
      ) : null}

      {/* Context menu sheet */}
      <FileActionsSheet menu={menu} onClose={() => setMenu(null)} />
    </SafeAreaView>
  );
}
