import { Ionicons } from "@expo/vector-icons";
import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import {
  ActivityIndicator,
  Alert,
  BackHandler,
  FlatList,
  KeyboardAvoidingView,
  Modal,
  Platform,
  Pressable,
  RefreshControl,
  Text,
  TextInput,
  View,
} from "react-native";
import Svg, { Path } from "react-native-svg";
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

// ── MD5 (pure JS, RFC 1321) — required for Navidrome Subsonic auth ────────────

function md5(str: string): string {
  function safeAdd(x: number, y: number): number {
    const lsw = (x & 0xffff) + (y & 0xffff);
    const msw = (x >> 16) + (y >> 16) + (lsw >> 16);
    return (msw << 16) | (lsw & 0xffff);
  }
  function rol(n: number, c: number): number { return (n << c) | (n >>> (32 - c)); }
  function cmn(q: number, a: number, b: number, x: number, s: number, t: number): number {
    return safeAdd(rol(safeAdd(safeAdd(a, q), safeAdd(x, t)), s), b);
  }
  const ff = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) =>
    cmn((b & c) | (~b & d), a, b, x, s, t);
  const gg = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) =>
    cmn((b & d) | (c & ~d), a, b, x, s, t);
  const hh = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) =>
    cmn(b ^ c ^ d, a, b, x, s, t);
  const ii = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) =>
    cmn(c ^ (b | ~d), a, b, x, s, t);
  function s2b(s: string): number[] {
    const b: number[] = [];
    for (let i = 0; i < s.length * 8; i += 8)
      b[i >> 5] |= (s.charCodeAt(i / 8) & 0xff) << i % 32;
    return b;
  }
  function b2h(b: number[]): string {
    const h = "0123456789abcdef"; let s = "";
    for (let i = 0; i < b.length * 4; i++)
      s += h[(b[i >> 2] >> (i % 4) * 8 + 4) & 0xf] + h[(b[i >> 2] >> (i % 4) * 8) & 0xf];
    return s;
  }
  function core(x: number[], len: number): number[] {
    x[len >> 5] |= 0x80 << len % 32;
    x[(((len + 64) >>> 9) << 4) + 14] = len;
    let a = 1732584193, b = -271733879, c = -1732584194, d = 271733878;
    for (let i = 0; i < x.length; i += 16) {
      const [oa, ob, oc, od] = [a, b, c, d];
      a = ff(a,b,c,d,x[i],7,-680876936); d=ff(d,a,b,c,x[i+1],12,-389564586); c=ff(c,d,a,b,x[i+2],17,606105819); b=ff(b,c,d,a,x[i+3],22,-1044525330);
      a = ff(a,b,c,d,x[i+4],7,-176418897); d=ff(d,a,b,c,x[i+5],12,1200080426); c=ff(c,d,a,b,x[i+6],17,-1473231341); b=ff(b,c,d,a,x[i+7],22,-45705983);
      a = ff(a,b,c,d,x[i+8],7,1770035416); d=ff(d,a,b,c,x[i+9],12,-1958414417); c=ff(c,d,a,b,x[i+10],17,-42063); b=ff(b,c,d,a,x[i+11],22,-1990404162);
      a = ff(a,b,c,d,x[i+12],7,1804603682); d=ff(d,a,b,c,x[i+13],12,-40341101); c=ff(c,d,a,b,x[i+14],17,-1502002290); b=ff(b,c,d,a,x[i+15],22,1236535329);
      a = gg(a,b,c,d,x[i+1],5,-165796510); d=gg(d,a,b,c,x[i+6],9,-1069501632); c=gg(c,d,a,b,x[i+11],14,643717713); b=gg(b,c,d,a,x[i],20,-373897302);
      a = gg(a,b,c,d,x[i+5],5,-701558691); d=gg(d,a,b,c,x[i+10],9,38016083); c=gg(c,d,a,b,x[i+15],14,-660478335); b=gg(b,c,d,a,x[i+4],20,-405537848);
      a = gg(a,b,c,d,x[i+9],5,568446438); d=gg(d,a,b,c,x[i+14],9,-1019803690); c=gg(c,d,a,b,x[i+3],14,-187363961); b=gg(b,c,d,a,x[i+8],20,1163531501);
      a = gg(a,b,c,d,x[i+13],5,-1444681467); d=gg(d,a,b,c,x[i+2],9,-51403784); c=gg(c,d,a,b,x[i+7],14,1735328473); b=gg(b,c,d,a,x[i+12],20,-1926607734);
      a = hh(a,b,c,d,x[i+5],4,-378558); d=hh(d,a,b,c,x[i+8],11,-2022574463); c=hh(c,d,a,b,x[i+11],16,1839030562); b=hh(b,c,d,a,x[i+14],23,-35309556);
      a = hh(a,b,c,d,x[i+1],4,-1530992060); d=hh(d,a,b,c,x[i+4],11,1272893353); c=hh(c,d,a,b,x[i+7],16,-155497632); b=hh(b,c,d,a,x[i+10],23,-1094730640);
      a = hh(a,b,c,d,x[i+13],4,681279174); d=hh(d,a,b,c,x[i],11,-358537222); c=hh(c,d,a,b,x[i+3],16,-722521979); b=hh(b,c,d,a,x[i+6],23,76029189);
      a = hh(a,b,c,d,x[i+9],4,-640364487); d=hh(d,a,b,c,x[i+12],11,-421815835); c=hh(c,d,a,b,x[i+15],16,530742520); b=hh(b,c,d,a,x[i+2],23,-995338651);
      a = ii(a,b,c,d,x[i],6,-198630844); d=ii(d,a,b,c,x[i+7],10,1126891415); c=ii(c,d,a,b,x[i+14],15,-1416354905); b=ii(b,c,d,a,x[i+5],21,-57434055);
      a = ii(a,b,c,d,x[i+12],6,1700485571); d=ii(d,a,b,c,x[i+3],10,-1894986606); c=ii(c,d,a,b,x[i+10],15,-1051523); b=ii(b,c,d,a,x[i+1],21,-2054922799);
      a = ii(a,b,c,d,x[i+8],6,1873313359); d=ii(d,a,b,c,x[i+15],10,-30611744); c=ii(c,d,a,b,x[i+6],15,-1560198380); b=ii(b,c,d,a,x[i+13],21,1309151649);
      a = ii(a,b,c,d,x[i+4],6,-145523070); d=ii(d,a,b,c,x[i+11],10,-1120210379); c=ii(c,d,a,b,x[i+2],15,718787259); b=ii(b,c,d,a,x[i+9],21,-343485551);
      a=safeAdd(a,oa); b=safeAdd(b,ob); c=safeAdd(c,oc); d=safeAdd(d,od);
    }
    return [a,b,c,d];
  }
  const enc = unescape(encodeURIComponent(str));
  return b2h(core(s2b(enc), enc.length * 8));
}

function randomSalt(): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  return Array.from({ length: 10 }, () => chars[Math.floor(Math.random() * chars.length)]).join("");
}

// ── Insert position constants (matches apps/playlist.h) ───────────────────────

const INSERT_NEXT = -4;
const INSERT_LAST = -3;
const INSERT_SHUFFLED = -5;
const INSERT_LAST_SHUFFLED = -7;

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
  displayLabel: string | null;
  history: { mode: FilesMode; path: string | null; displayLabel: string | null }[];
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

function headerLabel(mode: FilesMode, _path: string | null, displayLabel: string | null): string {
  if (displayLabel) return displayLabel;
  switch (mode) {
    case "root": return "Files";
    case "local": return "Music";
    case "upnp-devices": return "UPnP Devices";
    case "upnp-browse": return "UPnP";
    case "plex-servers": return "Plex";
    case "plex-browse": return "Plex";
    case "jellyfin-servers": return "Jellyfin";
    case "jellyfin-browse": return "Jellyfin";
    case "navidrome-browse": return "Navidrome";
    case "kodi-servers": return "Kodi";
    case "kodi-browse": return "Kodi";
    default: return "Files";
  }
}

// ── Fetch path ────────────────────────────────────────────────────────────────

function fetchPath(mode: FilesMode, path: string | null): string | null {
  switch (mode) {
    case "local": return path;
    case "upnp-devices": return "upnp://";
    case "upnp-browse": return path;
    case "plex-servers": return "plex://";
    case "plex-browse": return path;
    case "jellyfin-servers": return "jellyfin://";
    case "jellyfin-browse": return path;
    case "navidrome-browse": return path;
    case "kodi-servers": return "kodi://";
    case "kodi-browse": return path;
    default: return null;
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function isServerListMode(mode: FilesMode): boolean {
  return (
    mode === "plex-servers" ||
    mode === "jellyfin-servers" ||
    mode === "kodi-servers" ||
    mode === "upnp-devices"
  );
}

// ── Context menu sheet ────────────────────────────────────────────────────────

type MenuEntry = { entry: FileEntry; currentDir: string | null; entryIndex: number };

function FileActionsSheet({ menu, onClose }: { menu: MenuEntry | null; onClose: () => void }) {
  const { queue } = usePlayer();

  if (!menu) {
    return <Modal visible={false} transparent animationType="slide" onRequestClose={onClose} />;
  }
  const { entry, currentDir, entryIndex } = menu;
  const isDir = entry.is_dir;

  const run = (action: () => Promise<unknown>) => { action().catch(() => {}); onClose(); };
  const confirmPlay = (action: () => void) => {
    if (queue.length > 0) {
      Alert.alert("Replace Queue", "This will clear the current queue.", [
        { text: "Cancel", style: "cancel" },
        { text: "Play", style: "destructive", onPress: action },
      ]);
    } else { action(); }
    onClose();
  };

  type MenuItem = { icon: React.ComponentProps<typeof Ionicons>["name"]; label: string; separator?: boolean; onPress: () => void };
  const items: MenuItem[] = [
    { icon: "play-circle-outline", label: "Play", onPress: () => confirmPlay(() => (isDir ? RockboxClient.playDirectory(entry.path, false) : RockboxClient.playDirectory(currentDir ?? entry.path, false, entryIndex)).catch(() => {})) },
    { icon: "play-skip-forward-outline", label: "Play Next", onPress: () => run(() => isDir ? RockboxClient.insertDirectory(entry.path, INSERT_NEXT) : RockboxClient.insertTrackNext(entry.path)) },
    { icon: "list-outline", label: "Play Last", onPress: () => run(() => isDir ? RockboxClient.insertDirectory(entry.path, INSERT_LAST) : RockboxClient.insertTrackLast(entry.path)) },
    { icon: "shuffle-outline", label: "Add Shuffled", onPress: () => run(() => isDir ? RockboxClient.insertDirectory(entry.path, INSERT_SHUFFLED) : RockboxClient.insertTracks([entry.path], INSERT_SHUFFLED, false)) },
    ...(isDir ? ([
      { icon: "shuffle-outline" as const, label: "Play Last Shuffled", separator: true, onPress: () => run(() => RockboxClient.insertDirectory(entry.path, INSERT_LAST_SHUFFLED)) },
      { icon: "play-circle" as const, label: "Play Shuffled", onPress: () => confirmPlay(() => RockboxClient.playDirectory(entry.path, true).catch(() => {})) },
    ] as MenuItem[]) : []),
  ];

  return (
    <Modal visible transparent animationType="slide" onRequestClose={onClose}>
      <Pressable onPress={onClose} className="flex-1 bg-black/55">
        <Pressable onPress={(e) => e.stopPropagation()} className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7">
          <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
          <View className="flex-row items-center gap-3 px-4 py-3 border-b border-divider">
            <View className="w-10 h-10 rounded-lg bg-bg-card items-center justify-center shrink-0">
              {isDir ? <Ionicons name="folder-outline" size={22} color={Colors.textMuted} /> : <Ionicons name="musical-note" size={22} color={Colors.textMuted} />}
            </View>
            <Text numberOfLines={2} className="flex-1 text-text-primary text-[15px] font-display-medium">{entry.name}</Text>
          </View>
          {items.map((item) => (
            <View key={item.label}>
              {item.separator ? <View className="h-px bg-divider mx-4 my-1" /> : null}
              <Pressable onPress={item.onPress} android_ripple={{ color: Colors.bgHover }} className="flex-row items-center gap-4 px-5 py-3.5 active:bg-bg-hover">
                <Ionicons name={item.icon} size={22} color={Colors.textPrimary} />
                <Text className="text-text-primary text-[15px] font-sans">{item.label}</Text>
              </Pressable>
            </View>
          ))}
        </Pressable>
      </Pressable>
    </Modal>
  );
}

// ── Root tiles ────────────────────────────────────────────────────────────────

function RootTile({ icon, label, onPress }: { icon: React.ReactNode; label: string; onPress: () => void }) {
  return (
    <Pressable onPress={onPress} className="flex-1 bg-bg-card rounded-xl p-5 gap-3 active:opacity-70">
      {icon}
      <Text className="text-text-primary text-[15px] font-display-medium">{label}</Text>
    </Pressable>
  );
}

// ── Server / device row ───────────────────────────────────────────────────────

function ServerRow({ entry, icon, onPress }: { entry: FileEntry; icon: React.ReactNode; onPress: (entry: FileEntry) => void }) {
  return (
    <Pressable onPress={() => onPress(entry)} className="flex-row items-center px-4 py-3 gap-3 active:bg-bg-card">
      <View className="w-9 h-9 rounded-lg bg-bg-card items-center justify-center shrink-0">
        {icon}
      </View>
      <Text className="flex-1 text-text-primary text-[14px] font-sans" numberOfLines={1}>{entry.name}</Text>
      <Ionicons name="chevron-forward" size={16} color={Colors.textMuted} />
    </Pressable>
  );
}

// ── File row ──────────────────────────────────────────────────────────────────

function FileRow({ entry, index, currentDir, onNavigate, onPlay, onLongPress }: {
  entry: FileEntry; index: number; currentDir: string | null;
  onNavigate: (entry: FileEntry) => void;
  onPlay: (entry: FileEntry, index: number, dir: string | null) => void;
  onLongPress: (entry: FileEntry, index: number, dir: string | null) => void;
}) {
  return (
    <Pressable
      onPress={() => entry.is_dir ? onNavigate(entry) : onPlay(entry, index, currentDir)}
      onLongPress={() => onLongPress(entry, index, currentDir)}
      className="flex-row items-center px-4 py-3 gap-3 active:bg-bg-card"
    >
      <View className="w-9 h-9 rounded-lg bg-bg-card items-center justify-center shrink-0">
        {entry.is_dir ? <Ionicons name="folder-outline" size={20} color={Colors.textMuted} /> : <Ionicons name="musical-note" size={20} color={Colors.textMuted} />}
      </View>
      <Text className="flex-1 text-text-primary text-[14px] font-sans" numberOfLines={1}>{entry.name}</Text>
      <Pressable hitSlop={8} onPress={() => onLongPress(entry, index, currentDir)}>
        <Ionicons name="ellipsis-horizontal" size={18} color={Colors.textMuted} />
      </Pressable>
    </Pressable>
  );
}

// ── Connect modal ─────────────────────────────────────────────────────────────

function ConnectModal({
  visible, title, onClose, children,
}: {
  visible: boolean; title: string; onClose: () => void; children: React.ReactNode;
}) {
  return (
    <Modal visible={visible} transparent animationType="fade" onRequestClose={onClose}>
      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : "height"}
        className="flex-1"
      >
        <Pressable onPress={onClose} className="flex-1 bg-black/60 items-center justify-center px-5">
          <Pressable onPress={(e) => e.stopPropagation()} className="w-full bg-bg-elevated rounded-2xl p-6 gap-4">
            <Text className="text-text-primary text-[18px] font-display-medium">{title}</Text>
            {children}
          </Pressable>
        </Pressable>
      </KeyboardAvoidingView>
    </Modal>
  );
}

function FormInput({ value, onChangeText, placeholder, secureTextEntry = false, autoCapitalize = "none", keyboardType = "default" }: {
  value: string; onChangeText: (v: string) => void; placeholder: string;
  secureTextEntry?: boolean; autoCapitalize?: "none" | "sentences" | "words" | "characters";
  keyboardType?: "default" | "url" | "email-address";
}) {
  return (
    <TextInput
      value={value}
      onChangeText={onChangeText}
      placeholder={placeholder}
      placeholderTextColor={Colors.textMuted}
      secureTextEntry={secureTextEntry}
      autoCapitalize={autoCapitalize}
      autoCorrect={false}
      keyboardType={keyboardType}
      className="bg-bg-card rounded-xl px-4 py-3 text-text-primary text-[14px] font-sans"
    />
  );
}

function ConnectButton({ label, onPress, loading = false, disabled = false }: {
  label: string; onPress: () => void; loading?: boolean; disabled?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled || loading}
      className="bg-accent rounded-xl py-3.5 items-center active:opacity-80"
      style={{ opacity: disabled || loading ? 0.5 : 1 }}
    >
      {loading ? (
        <ActivityIndicator size="small" color="#fff" />
      ) : (
        <Text className="text-white text-[15px] font-display-medium">{label}</Text>
      )}
    </Pressable>
  );
}

// ── SVG icons for media servers ───────────────────────────────────────────────

function PlexIcon({ size = 28, color = Colors.textMuted }: { size?: number; color?: string }) {
  return (
    <Svg width={size} height={size} viewBox="0 0 24 24" fill={color}>
      <Path d="M12 0C5.372 0 0 5.372 0 12s5.372 12 12 12 12-5.372 12-12S18.628 0 12 0zm1.492 18.642H8.866V5.358h4.626c2.62 0 4.28 2.196 4.28 4.592 0 1.75-.69 3.146-2.056 3.99.813.57 1.326 1.47 1.326 2.55 0 1.453-.908 2.152-3.55 2.152z" />
    </Svg>
  );
}

function JellyfinIcon({ size = 28, color = Colors.textMuted }: { size?: number; color?: string }) {
  return (
    <Svg width={size} height={size} viewBox="0 0 24 24" fill={color}>
      <Path d="M12 0.002C8.826 0.002 -1.398 18.537 0.16 21.666c1.56 3.129 22.14 3.094 23.682 0C25.384 18.573 15.177 0 12 0zm7.76 18.949c-1.008 2.028 -14.493 2.05 -15.514 0C3.224 16.9 9.92 4.755 12.003 4.755c2.081 0 8.77 12.166 7.759 14.196zM12 9.198c-1.054 0 -4.446 6.15 -3.93 7.189 0.518 1.04 7.348 1.027 7.86 0 0.511 -1.027 -2.874 -7.19 -3.93 -7.19z" />
    </Svg>
  );
}

function NavidromeIcon({ size = 28, color = Colors.textMuted }: { size?: number; color?: string }) {
  return (
    <Svg width={size} height={size} viewBox="0 0 24 24" fill={color}>
      <Path d="M12 0C5.36692 0 0 5.36697 0 12c0 6.63308 5.36692 12 12 12s12-5.36692 12-12c0-6.63303-5.36692-12-12-12Zm0 2.04242c.28204 0 .51111.22908.51111.51112s-.22907.50908-.51111.50908c-4.23974 0-7.92423 3.0107-8.76022 7.15764-.04887.2424-.2608.4093-.4989.4093a.51578.51578 0 0 1-.10182-.01018c-.27646-.05571-.45483-.32425-.39911-.60071.44785-2.2215 1.66041-4.24424 3.4149-5.69557C7.43279 2.8523 9.68579 2.04242 12 2.04242Zm0 2.29697c.28204 0 .51111.22907.51111.51111 0 .28204-.22907.51112-.51111.51112-3.66047 0-6.63838 2.9779-6.63838 6.63838 0 .28204-.22908.51111-.51112.51111S4.3394 12.28205 4.3394 12c0-2.04757.798-3.97064 2.24401-5.4166C8.02941 5.13735 9.95243 4.3394 12 4.3394Zm0 2.86102c2.64683 0 4.7996 2.15281 4.7996 4.79959 0 2.64683-2.15282 4.7996-4.7996 4.7996-2.64678 0-4.7996-2.15282-4.7996-4.7996 0-2.64678 2.15277-4.7996 4.7996-4.7996Zm0 1.0202C9.91632 8.2206 8.2206 9.9163 8.2206 12c0 2.08368 1.69572 3.7794 3.7794 3.7794S15.7794 14.08368 15.7794 12 14.08368 8.2206 12 8.2206zm0 2.3499c.78844 0 1.4295.6411 1.4295 1.42949 0 .78844-.6411 1.4295-1.4295 1.4295S10.5705 12.7884 10.5705 12s.64106-1.4295 1.4295-1.4295zm-9.4322.6903c.28193.0067.50554.24135.49889.52334-.00166.07194-.00407.14347-.00407.21585 0 .28204-.22704.51111-.50908.51111-.28204 0-.51111-.22907-.51111-.51111 0-.08044.00013-.16028.00204-.24029.0066-.28193.24188-.50573.52333-.4989zm16.5817.22807c.28204 0 .51111.22908.51111.51112 0 2.04757-.79805 3.97064-2.24401 5.4166C15.97058 18.86264 14.04756 19.6606 12 19.6606c-.28204 0-.51112-.22907-.51112-.51111 0-.28204.22908-.51112.51112-.51112 3.66047 0 6.63843-2.9779 6.63838-6.63838 0-.28204.22908-.51112.51112-.51112zm2.29696 0c.28204 0 .51102.22918.51112.51112 0 .08044-.00015.16028-.002.24028-.0066.27784-.2347.4989-.51112.4989-.0041 0-.0081.0001-.01222 0-.28194-.0066-.50554-.2414-.4989-.52333.0017-.07194.0041-.14347.0041-.21585 0-.28204.22705-.51112.50908-.51112zM12 11.5907c-.22525 0-.4093.18405-.4093.4093s.18405.4093.4093.4093.4093-.18405.4093-.4093-.18405-.4093-.4093-.4093zm9.25912 1.77974a.50122.50122 0 0 1 .10182.01018c.27646.05576.45483.3243.39911.60071-.44785 2.2215-1.66041 4.24424-3.4149 5.69557C16.56721 21.1477 14.31421 21.95758 12 21.95758c-.28204 0-.51111-.22908-.51111-.51112s.22907-.50908.51111-.50908c4.23974 0 7.92418-3.0107 8.76022-7.15764.04875-.24195.2614-.40872.4989-.4093z" />
    </Svg>
  );
}

function KodiIcon({ size = 28, color = Colors.textMuted }: { size?: number; color?: string }) {
  return (
    <Svg width={size} height={size} viewBox="0 0 24 24" fill={color}>
      <Path d="M12.03.047c-.226 0-.452.107-.669.324-.922.922-1.842 1.845-2.763 2.768-.233.233-.455.48-.703.695-.31.267-.405.583-.399.988.02 1.399.008 2.799.008 4.198 0 1.453-.002 2.907 0 4.36 0 .11.002.223.03.327.087.337.303.393.546.15 1.31-1.31 2.618-2.622 3.928-3.933l4.449-4.453c.43-.431.43-.905 0-1.336L12.697.37c-.216-.217-.442-.324-.668-.324zm7.224 7.23c-.223 0-.445.104-.65.309L14.82 11.37c-.428.429-.427.895 0 1.322l3.76 3.766c.44.44.908.44 1.346.002 1.215-1.216 2.427-2.433 3.644-3.647.182-.18.353-.364.43-.615v-.33c-.077-.251-.246-.436-.428-.617-1.224-1.22-2.443-2.445-3.666-3.668-.205-.205-.429-.307-.652-.307zM4.18 7.611c-.086.014-.145.094-.207.157L.209 11.572c-.28.284-.278.677.004.96l2.043 2.046c.59.59 1.177 1.182 1.767 1.772.169.168.33.139.416-.084.044-.114.062-.242.063-.364.004-1.283.004-2.567.004-3.851h-.002V8.184c0-.085-.01-.169-.022-.252-.019-.135-.072-.258-.207-.309a.186.186 0 0 0-.095-.012zm7.908 6.838c-.224 0-.447.106-.656.315L7.66 18.537c-.433.434-.433.899.002 1.334 1.215 1.216 2.43 2.43 3.643 3.649.18.18.361.354.611.433h.33c.244-.069.423-.226.598-.402 1.222-1.23 2.45-2.453 3.676-3.68.43-.43.427-.905-.004-1.338l-3.772-3.773c-.208-.208-.432-.311-.656-.31z" />
    </Svg>
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
  const cache = useRef<Map<string, FileEntry[]>>(new Map());

  // ── Plex state ──────────────────────────────────────────────────────────────
  const [showPlexAuth, setShowPlexAuth] = useState(false);
  const [pendingPlexServer, setPendingPlexServer] = useState<string | null>(null);
  const [plexToken, setPlexToken] = useState("");
  const [showPlexManual, setShowPlexManual] = useState(false);
  const [plexManualUrl, setPlexManualUrl] = useState("");

  // ── Jellyfin state ──────────────────────────────────────────────────────────
  const [showJellyfinAuth, setShowJellyfinAuth] = useState(false);
  const [pendingJellyfinServer, setPendingJellyfinServer] = useState<string | null>(null);
  const [jellyfinUsername, setJellyfinUsername] = useState("");
  const [jellyfinPassword, setJellyfinPassword] = useState("");
  const [jellyfinError, setJellyfinError] = useState<string | null>(null);
  const [jellyfinSigningIn, setJellyfinSigningIn] = useState(false);
  const [jellyfinAuthMode, setJellyfinAuthMode] = useState<"credentials" | "apikey">("credentials");
  const [jellyfinApiKey, setJellyfinApiKey] = useState("");
  const [showJellyfinManual, setShowJellyfinManual] = useState(false);
  const [jellyfinManualUrl, setJellyfinManualUrl] = useState("");

  // ── Navidrome state ─────────────────────────────────────────────────────────
  const [showNavidromeEntry, setShowNavidromeEntry] = useState(false);
  const [navidromeUrl, setNavidromeUrl] = useState("");
  const [navidromeUsername, setNavidromeUsername] = useState("");
  const [navidromePassword, setNavidromePassword] = useState("");
  const [navidromeError, setNavidromeError] = useState<string | null>(null);
  const [navidromeConnecting, setNavidromeConnecting] = useState(false);

  // ── Kodi state ──────────────────────────────────────────────────────────────
  const [showKodiAuth, setShowKodiAuth] = useState(false);
  const [kodiUrl, setKodiUrl] = useState("");
  const [kodiUsername, setKodiUsername] = useState("");
  const [kodiPassword, setKodiPassword] = useState("");
  const [kodiError, setKodiError] = useState<string | null>(null);

  // ── Fetch ───────────────────────────────────────────────────────────────────

  const canGoBack = browse.history.length > 0;
  const label = headerLabel(browse.mode, browse.path, browse.displayLabel);

  const doFetch = useCallback(
    (mode: FilesMode, path: string | null, isRefresh: boolean) => {
      if (!isConnected) return;
      const fp = fetchPath(mode, path);
      if (fp === null && mode !== "local") return;
      const seq = ++fetchSeq.current;
      const key = `${mode}:${path ?? ""}`;
      const cached = cache.current.get(key);

      if (isRefresh) setRefreshing(true);
      else if (cached) setEntries(cached);
      else setLoading(true);

      RockboxClient.treeGetEntries(fp)
        .then((res: unknown) => {
          if (fetchSeq.current !== seq) return;
          const raw = (res as { entries?: RawEntry[] })?.entries ?? [];
          const sorted = sortEntries(raw.map(toFileEntry));
          cache.current.set(key, sorted);
          setEntries(sorted);
        })
        .catch(() => {
          if (fetchSeq.current !== seq) return;
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
    if (browse.mode === "root") { setEntries([]); return; }
    doFetch(browse.mode, browse.path, false);
  }, [browse.mode, browse.path, doFetch]);

  const handleRefresh = useCallback(() => doFetch(browse.mode, browse.path, true), [browse.mode, browse.path, doFetch]);

  const navigate = useCallback(
    (mode: FilesMode, path: string | null, displayLabel?: string) =>
      dispatch({ type: "push", mode, path, displayLabel }),
    [],
  );

  const goBack = useCallback(() => dispatch({ type: "pop" }), []);

  useFocusEffect(
    useCallback(() => {
      const sub = BackHandler.addEventListener("hardwareBackPress", () => {
        if (browse.history.length > 0) { dispatch({ type: "pop" }); return true; }
        return false;
      });
      return () => sub.remove();
    }, [browse.history.length]),
  );

  // ── Navigation handler ──────────────────────────────────────────────────────

  const handleNavigate = useCallback(
    (entry: FileEntry) => {
      const { mode } = browse;
      if (mode === "plex-servers") {
        setPendingPlexServer(entry.path);
        setPlexToken("");
        setShowPlexAuth(true);
      } else if (mode === "jellyfin-servers") {
        setPendingJellyfinServer(entry.path);
        setJellyfinUsername("");
        setJellyfinPassword("");
        setJellyfinApiKey("");
        setJellyfinError(null);
        setJellyfinAuthMode("credentials");
        setShowJellyfinAuth(true);
      } else if (mode === "kodi-servers") {
        navigate("kodi-browse", entry.path, entry.name);
      } else if (mode === "upnp-devices" || mode === "upnp-browse") {
        navigate("upnp-browse", entry.path, entry.name);
      } else if (mode === "plex-browse") {
        navigate("plex-browse", entry.path, entry.name);
      } else if (mode === "jellyfin-browse") {
        navigate("jellyfin-browse", entry.path, entry.name);
      } else if (mode === "navidrome-browse") {
        navigate("navidrome-browse", entry.path, entry.name);
      } else if (mode === "kodi-browse") {
        navigate("kodi-browse", entry.path, entry.name);
      } else {
        navigate("local", entry.path, entry.name);
      }
    },
    [browse, navigate],
  );

  const handlePlay = useCallback(
    (entry: FileEntry, index: number, dir: string | null) => {
      if (isServerListMode(browse.mode)) return;
      const currentDir = dir ?? inferCurrentDir(entries);
      const doPlay = () => RockboxClient.playDirectory(currentDir ?? entry.path, false, index).catch(() => {});
      if (queue.length > 0) {
        Alert.alert("Replace Queue", "This will clear the current queue.", [
          { text: "Cancel", style: "cancel" },
          { text: "Play", style: "destructive", onPress: doPlay },
        ]);
      } else doPlay();
    },
    [entries, queue.length, browse.mode],
  );

  const openMenu = useCallback(
    (entry: FileEntry, index: number, dir: string | null) => {
      if (isServerListMode(browse.mode)) return;
      setMenu({ entry, currentDir: dir ?? inferCurrentDir(entries), entryIndex: index });
    },
    [entries, browse.mode],
  );

  const handleAddManually = useCallback(() => {
    if (browse.mode === "plex-servers") { setPlexManualUrl(""); setShowPlexManual(true); }
    else if (browse.mode === "jellyfin-servers") { setJellyfinManualUrl(""); setShowJellyfinManual(true); }
    else if (browse.mode === "kodi-servers") { setKodiUrl(""); setKodiUsername(""); setKodiPassword(""); setKodiError(null); setShowKodiAuth(true); }
  }, [browse]);

  // ── Connect handlers ────────────────────────────────────────────────────────

  const handlePlexConnect = useCallback(() => {
    if (!pendingPlexServer) return;
    const navPath = plexToken.trim()
      ? `${pendingPlexServer}%3FX-Plex-Token%3D${encodeURIComponent(plexToken.trim())}`
      : pendingPlexServer;
    navigate("plex-browse", navPath, "Plex");
    setShowPlexAuth(false);
    setPendingPlexServer(null);
    setPlexToken("");
  }, [pendingPlexServer, plexToken, navigate]);

  const handlePlexManualAdd = useCallback(() => {
    const url = plexManualUrl.trim().replace(/\/$/, "");
    if (!url) return;
    setShowPlexManual(false);
    setPendingPlexServer(`plex://${encodeURIComponent(url)}`);
    setPlexToken("");
    setShowPlexAuth(true);
  }, [plexManualUrl]);

  const handleJellyfinSignIn = useCallback(async () => {
    if (!pendingJellyfinServer) return;
    const encoded = pendingJellyfinServer.slice("jellyfin://".length).split("/")[0];
    const baseUrl = decodeURIComponent(encoded).split("?")[0];
    setJellyfinSigningIn(true);
    setJellyfinError(null);
    try {
      const resp = await fetch(`${baseUrl.replace(/\/$/, "")}/Users/AuthenticateByName`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Emby-Authorization": 'MediaBrowser Client="Rockbox", Device="Mobile", DeviceId="rockbox-mobile", Version="1.0"',
        },
        body: JSON.stringify({ Username: jellyfinUsername, Pw: jellyfinPassword }),
      });
      if (resp.ok) {
        const data = await resp.json();
        const token: string = data.AccessToken;
        const userId: string = data.User.Id;
        const navPath = `${pendingJellyfinServer}%3FX-Jellyfin-Token%3D${encodeURIComponent(token)}%26userId%3D${encodeURIComponent(userId)}`;
        navigate("jellyfin-browse", navPath, "Jellyfin");
        setShowJellyfinAuth(false);
        setPendingJellyfinServer(null);
      } else {
        setJellyfinError("Authentication failed. Check username/password.");
      }
    } catch {
      setJellyfinError("Could not reach the Jellyfin server.");
    } finally {
      setJellyfinSigningIn(false);
    }
  }, [pendingJellyfinServer, jellyfinUsername, jellyfinPassword, navigate]);

  const handleJellyfinApiKey = useCallback(async () => {
    if (!pendingJellyfinServer) return;
    const encoded = pendingJellyfinServer.slice("jellyfin://".length).split("/")[0];
    const baseUrl = decodeURIComponent(encoded).split("?")[0];
    setJellyfinSigningIn(true);
    setJellyfinError(null);
    try {
      const resp = await fetch(`${baseUrl.replace(/\/$/, "")}/Users`, {
        headers: { "X-Emby-Token": jellyfinApiKey },
      });
      if (resp.ok) {
        const users = await resp.json();
        const userId: string = users[0]?.Id;
        if (!userId) { setJellyfinError("No users found for this API key."); return; }
        const navPath = `${pendingJellyfinServer}%3FX-Jellyfin-Token%3D${encodeURIComponent(jellyfinApiKey)}%26userId%3D${encodeURIComponent(userId)}`;
        navigate("jellyfin-browse", navPath, "Jellyfin");
        setShowJellyfinAuth(false);
        setPendingJellyfinServer(null);
      } else {
        setJellyfinError("Invalid API key or insufficient permissions.");
      }
    } catch {
      setJellyfinError("Could not reach the Jellyfin server.");
    } finally {
      setJellyfinSigningIn(false);
    }
  }, [pendingJellyfinServer, jellyfinApiKey, navigate]);

  const handleJellyfinManualConnect = useCallback(() => {
    const url = jellyfinManualUrl.trim().replace(/\/$/, "");
    if (!url) return;
    setShowJellyfinManual(false);
    setPendingJellyfinServer(`jellyfin://${encodeURIComponent(url)}`);
    setJellyfinUsername(""); setJellyfinPassword(""); setJellyfinError(null);
    setJellyfinAuthMode("credentials");
    setShowJellyfinAuth(true);
  }, [jellyfinManualUrl]);

  const handleNavidromeConnect = useCallback(async () => {
    const baseUrl = navidromeUrl.trim().replace(/\/$/, "");
    if (!baseUrl || !navidromeUsername.trim()) return;
    setNavidromeConnecting(true);
    setNavidromeError(null);
    try {
      const salt = randomSalt();
      const token = md5(navidromePassword + salt);
      const pingUrl = `${baseUrl}/rest/ping.view?u=${encodeURIComponent(navidromeUsername)}&t=${token}&s=${salt}&v=1.16.1&c=rockbox&f=json`;
      const resp = await fetch(pingUrl);
      if (!resp.ok) throw new Error("HTTP " + resp.status);
      const json = await resp.json();
      if (json["subsonic-response"]?.status !== "ok") {
        setNavidromeError("Authentication failed. Check username/password.");
        return;
      }
      const credUrl = `${baseUrl}?nd_user=${encodeURIComponent(navidromeUsername)}&nd_token=${token}&nd_salt=${salt}`;
      navigate("navidrome-browse", `navidrome://${encodeURIComponent(credUrl)}`, "Navidrome");
      setShowNavidromeEntry(false);
      setNavidromeUrl(""); setNavidromeUsername(""); setNavidromePassword("");
    } catch {
      setNavidromeError("Could not reach the Navidrome server.");
    } finally {
      setNavidromeConnecting(false);
    }
  }, [navidromeUrl, navidromeUsername, navidromePassword, navigate]);

  const handleKodiConnect = useCallback(() => {
    const baseUrl = kodiUrl.trim().replace(/\/$/, "");
    if (!baseUrl) return;
    const credSuffix = kodiUsername.trim()
      ? `?kodi_user=${encodeURIComponent(kodiUsername)}&kodi_pass=${encodeURIComponent(kodiPassword)}`
      : "";
    navigate("kodi-browse", `kodi://${encodeURIComponent(`${baseUrl}${credSuffix}`)}`, "Kodi");
    setShowKodiAuth(false);
    setKodiUrl(""); setKodiUsername(""); setKodiPassword(""); setKodiError(null);
  }, [kodiUrl, kodiUsername, kodiPassword, navigate]);

  // ── Server list mode icon ───────────────────────────────────────────────────

  const serverIcon = (mode: FilesMode) => {
    if (mode === "plex-servers") return <PlexIcon size={20} />;
    if (mode === "jellyfin-servers") return <JellyfinIcon size={20} />;
    if (mode === "kodi-servers") return <KodiIcon size={20} />;
    return <Ionicons name="laptop-outline" size={20} color={Colors.textMuted} />;
  };

  // ── Render ──────────────────────────────────────────────────────────────────

  const inServerListMode = isServerListMode(browse.mode);
  const emptyMessage =
    browse.mode === "upnp-devices" ? "No UPnP devices found" :
    browse.mode === "plex-servers" ? "No Plex servers found" :
    browse.mode === "jellyfin-servers" ? "No Jellyfin servers found" :
    browse.mode === "kodi-servers" ? "No Kodi servers found" :
    "No files here";

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <RemoteServerBanner />

      {/* Header */}
      <View className="flex-row items-center px-4 pt-2 pb-3 gap-3">
        {canGoBack ? (
          <Pressable onPress={goBack} hitSlop={10} className="w-8 h-8 rounded-full bg-bg-card items-center justify-center shrink-0 active:opacity-70">
            <Ionicons name="chevron-back" size={20} color={Colors.textPrimary} />
          </Pressable>
        ) : null}
        <Text numberOfLines={1} className={`flex-1 text-text-primary font-display-extra ${canGoBack ? "text-[20px]" : "text-[26px]"}`}>
          {label}
        </Text>
      </View>

      {/* Root tiles — 2-column grid */}
      {browse.mode === "root" ? (
        <View className="gap-3 px-4 pt-2">
          <View className="flex-row gap-3">
            <RootTile icon={<Ionicons name="folder-open-outline" size={32} color={Colors.textMuted} />} label="Music" onPress={() => navigate("local", null)} />
            <RootTile icon={<Ionicons name="laptop-outline" size={32} color={Colors.textMuted} />} label="UPnP" onPress={() => navigate("upnp-devices", null)} />
          </View>
          <View className="flex-row gap-3">
            <RootTile icon={<PlexIcon size={32} />} label="Plex" onPress={() => navigate("plex-servers", null)} />
            <RootTile icon={<JellyfinIcon size={32} />} label="Jellyfin" onPress={() => navigate("jellyfin-servers", null)} />
          </View>
          <View className="flex-row gap-3">
            <RootTile
              icon={<NavidromeIcon size={32} />}
              label="Navidrome"
              onPress={() => {
                setNavidromeUrl(""); setNavidromeUsername(""); setNavidromePassword(""); setNavidromeError(null);
                setShowNavidromeEntry(true);
              }}
            />
            <RootTile icon={<KodiIcon size={32} />} label="Kodi" onPress={() => navigate("kodi-servers", null)} />
          </View>
        </View>
      ) : null}

      {/* Non-root: connection guard */}
      {browse.mode !== "root" && !isConnected ? (
        <View className="flex-1"><NotConnectedState /></View>
      ) : null}

      {/* Full-screen spinner */}
      {browse.mode !== "root" && isConnected && loading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator size="large" color={Colors.accent} />
        </View>
      ) : null}

      {/* File / server list */}
      {browse.mode !== "root" && isConnected && !loading ? (
        <FlatList
          data={entries}
          keyExtractor={(item) => item.path}
          contentContainerStyle={{ paddingBottom: bottomPad }}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
          }
          renderItem={({ item, index }) =>
            inServerListMode ? (
              <ServerRow entry={item} icon={serverIcon(browse.mode)} onPress={handleNavigate} />
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
          ListFooterComponent={
            inServerListMode && browse.mode !== "upnp-devices" ? (
              <Pressable onPress={handleAddManually} className="flex-row items-center px-4 py-3 gap-3 active:bg-bg-card">
                <View className="w-9 h-9 rounded-lg bg-bg-card items-center justify-center shrink-0">
                  <Ionicons name="add-outline" size={20} color={Colors.accent} />
                </View>
                <Text className="text-accent text-[14px] font-sans">Add Server Manually…</Text>
              </Pressable>
            ) : null
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <Ionicons name="server-outline" size={48} color={Colors.textMuted} />
              <Text className="text-text-muted text-[14px] mt-3">{emptyMessage}</Text>
            </View>
          }
        />
      ) : null}

      {/* Context menu */}
      <FileActionsSheet menu={menu} onClose={() => setMenu(null)} />

      {/* ── Plex: token modal ──────────────────────────────────────────────── */}
      <ConnectModal visible={showPlexAuth} title="Plex — Enter Token" onClose={() => { setShowPlexAuth(false); setPendingPlexServer(null); }}>
        <Text className="text-text-muted text-[13px] font-sans -mt-1">
          Your Plex token from plex.tv/claim or account settings.
        </Text>
        <FormInput value={plexToken} onChangeText={setPlexToken} placeholder="X-Plex-Token (optional)" />
        <ConnectButton
          label={plexToken.trim() ? "Browse with Token" : "Browse Without Token"}
          onPress={handlePlexConnect}
        />
      </ConnectModal>

      {/* ── Plex: manual URL modal ─────────────────────────────────────────── */}
      <ConnectModal visible={showPlexManual} title="Plex — Server URL" onClose={() => setShowPlexManual(false)}>
        <FormInput value={plexManualUrl} onChangeText={setPlexManualUrl} placeholder="http://192.168.1.x:32400" keyboardType="url" />
        <ConnectButton label="Continue" onPress={handlePlexManualAdd} disabled={!plexManualUrl.trim()} />
      </ConnectModal>

      {/* ── Jellyfin: credentials / API key modal ──────────────────────────── */}
      <ConnectModal
        visible={showJellyfinAuth}
        title="Jellyfin — Sign In"
        onClose={() => { setShowJellyfinAuth(false); setPendingJellyfinServer(null); setJellyfinError(null); }}
      >
        {/* Tab switcher */}
        <View className="flex-row bg-bg-card rounded-xl overflow-hidden">
          <Pressable
            onPress={() => setJellyfinAuthMode("credentials")}
            className="flex-1 py-2.5 items-center"
            style={{ backgroundColor: jellyfinAuthMode === "credentials" ? Colors.accent : "transparent" }}
          >
            <Text style={{ color: jellyfinAuthMode === "credentials" ? "#fff" : Colors.textMuted }} className="text-[13px] font-display-medium">
              Username
            </Text>
          </Pressable>
          <Pressable
            onPress={() => setJellyfinAuthMode("apikey")}
            className="flex-1 py-2.5 items-center"
            style={{ backgroundColor: jellyfinAuthMode === "apikey" ? Colors.accent : "transparent" }}
          >
            <Text style={{ color: jellyfinAuthMode === "apikey" ? "#fff" : Colors.textMuted }} className="text-[13px] font-display-medium">
              API Key
            </Text>
          </Pressable>
        </View>

        {jellyfinAuthMode === "credentials" ? (
          <>
            <FormInput value={jellyfinUsername} onChangeText={setJellyfinUsername} placeholder="Username" />
            <FormInput value={jellyfinPassword} onChangeText={setJellyfinPassword} placeholder="Password" secureTextEntry />
            <ConnectButton label="Sign In" onPress={handleJellyfinSignIn} loading={jellyfinSigningIn} disabled={!jellyfinUsername.trim()} />
          </>
        ) : (
          <>
            <FormInput value={jellyfinApiKey} onChangeText={setJellyfinApiKey} placeholder="API Key" />
            <ConnectButton label="Connect" onPress={handleJellyfinApiKey} loading={jellyfinSigningIn} disabled={!jellyfinApiKey.trim()} />
          </>
        )}

        {jellyfinError ? (
          <Text className="text-red-400 text-[13px] font-sans text-center">{jellyfinError}</Text>
        ) : null}
      </ConnectModal>

      {/* ── Jellyfin: manual URL modal ─────────────────────────────────────── */}
      <ConnectModal visible={showJellyfinManual} title="Jellyfin — Server URL" onClose={() => setShowJellyfinManual(false)}>
        <FormInput value={jellyfinManualUrl} onChangeText={setJellyfinManualUrl} placeholder="http://192.168.1.x:8096" keyboardType="url" />
        <ConnectButton label="Continue" onPress={handleJellyfinManualConnect} disabled={!jellyfinManualUrl.trim()} />
      </ConnectModal>

      {/* ── Navidrome: connect modal ───────────────────────────────────────── */}
      <ConnectModal
        visible={showNavidromeEntry}
        title="Navidrome"
        onClose={() => { setShowNavidromeEntry(false); setNavidromeError(null); }}
      >
        <FormInput value={navidromeUrl} onChangeText={setNavidromeUrl} placeholder="http://192.168.1.x:4533" keyboardType="url" />
        <FormInput value={navidromeUsername} onChangeText={setNavidromeUsername} placeholder="Username" />
        <FormInput value={navidromePassword} onChangeText={setNavidromePassword} placeholder="Password" secureTextEntry />
        <ConnectButton
          label="Connect"
          onPress={handleNavidromeConnect}
          loading={navidromeConnecting}
          disabled={!navidromeUrl.trim() || !navidromeUsername.trim()}
        />
        {navidromeError ? (
          <Text className="text-red-400 text-[13px] font-sans text-center">{navidromeError}</Text>
        ) : null}
      </ConnectModal>

      {/* ── Kodi: connect modal ────────────────────────────────────────────── */}
      <ConnectModal
        visible={showKodiAuth}
        title="Kodi — Server URL"
        onClose={() => { setShowKodiAuth(false); setKodiError(null); }}
      >
        <FormInput value={kodiUrl} onChangeText={setKodiUrl} placeholder="http://192.168.1.x:8080" keyboardType="url" />
        <Text className="text-text-muted text-[12px] font-sans -mt-2">
          Username and password are optional.
        </Text>
        <FormInput value={kodiUsername} onChangeText={setKodiUsername} placeholder="Username (optional)" />
        <FormInput value={kodiPassword} onChangeText={setKodiPassword} placeholder="Password (optional)" secureTextEntry />
        <ConnectButton label="Browse" onPress={handleKodiConnect} disabled={!kodiUrl.trim()} />
        {kodiError ? (
          <Text className="text-red-400 text-[13px] font-sans text-center">{kodiError}</Text>
        ) : null}
      </ConnectModal>
    </SafeAreaView>
  );
}
