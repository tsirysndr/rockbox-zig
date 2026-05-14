import { FC, useEffect, useRef, useState } from "react";
import Files from "./Files";
import {
  useGetEntriesQuery,
  usePlayDirectoryMutation,
} from "../../Hooks/GraphQL";
import { useNavigate, useSearchParams } from "react-router-dom";
import { File } from "../../Types/file";

const ROOT_ENTRIES: File[] = [
  { name: "Music", path: "__local__", isDirectory: true },
  { name: "UPnP Devices", path: "upnp://", isDirectory: true },
  { name: "Plex", path: "plex://", isDirectory: true },
  { name: "Jellyfin", path: "jellyfin://", isDirectory: true },
  { name: "Navidrome", path: "navidrome://", isDirectory: true },
  { name: "Kodi", path: "kodi://", isDirectory: true },
];

const JELLYFIN_ADD_MANUALLY: File = {
  name: "Add Server Manually…",
  path: "jellyfin://+manual",
  isDirectory: true,
};

/** True when a plex:// path refers to a server entry (no unencoded `/` after the scheme). */
function isPlexServerPath(path: string): boolean {
  const rest = path.slice("plex://".length);
  return rest.length > 0 && !rest.includes("/");
}

/** True when a jellyfin:// path refers to a server entry (no unencoded `/` after the scheme). */
function isJellyfinServerPath(path: string): boolean {
  const rest = path.slice("jellyfin://".length);
  return rest.length > 0 && !rest.includes("/");
}

/** True when a kodi:// path refers to a server entry (no unencoded `/` after the scheme). */
function isKodiServerPath(path: string): boolean {
  const rest = path.slice("kodi://".length);
  return rest.length > 0 && !rest.includes("/");
}

/** Compute MD5 hash (used for Subsonic token auth). Inline implementation — Web Crypto doesn't support MD5. */
function md5(input: string): string {
  const utf8 = unescape(encodeURIComponent(input));
  const bytes: number[] = [];
  for (let i = 0; i < utf8.length; i++) bytes.push(utf8.charCodeAt(i));
  const len8 = bytes.length;
  bytes.push(0x80);
  while (bytes.length % 64 !== 56) bytes.push(0);
  const b = len8 * 8;
  bytes.push(b & 0xff, (b >> 8) & 0xff, (b >> 16) & 0xff, (b >> 24) & 0xff, 0, 0, 0, 0);
  const M: number[] = [];
  for (let i = 0; i < bytes.length; i += 4)
    M.push(bytes[i] | (bytes[i + 1] << 8) | (bytes[i + 2] << 16) | (bytes[i + 3] << 24));
  const add = (x: number, y: number) => { const l = (x & 0xffff) + (y & 0xffff); return ((x >> 16) + (y >> 16) + (l >> 16)) << 16 | l & 0xffff; };
  const rol = (n: number, c: number) => n << c | n >>> (32 - c);
  const cmn = (q: number, a: number, b: number, x: number, s: number, t: number) => add(rol(add(add(a, q), add(x, t)), s), b);
  const ff = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(b & c | ~b & d, a, b, x, s, t);
  const gg = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(b & d | c & ~d, a, b, x, s, t);
  const hh = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(b ^ c ^ d, a, b, x, s, t);
  const ii = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(c ^ (b | ~d), a, b, x, s, t);
  let a = 0x67452301, bv = 0xefcdab89, c = 0x98badcfe, d = 0x10325476;
  for (let i = 0; i < M.length; i += 16) {
    const [aa, bb, cc, dd] = [a, bv, c, d];
    a=ff(a,bv,c,d,M[i+0],7,-680876936);d=ff(d,a,bv,c,M[i+1],12,-389564586);c=ff(c,d,a,bv,M[i+2],17,606105819);bv=ff(bv,c,d,a,M[i+3],22,-1044525330);
    a=ff(a,bv,c,d,M[i+4],7,-176418897);d=ff(d,a,bv,c,M[i+5],12,1200080426);c=ff(c,d,a,bv,M[i+6],17,-1473231341);bv=ff(bv,c,d,a,M[i+7],22,-45705983);
    a=ff(a,bv,c,d,M[i+8],7,1770035416);d=ff(d,a,bv,c,M[i+9],12,-1958414417);c=ff(c,d,a,bv,M[i+10],17,-42063);bv=ff(bv,c,d,a,M[i+11],22,-1990404162);
    a=ff(a,bv,c,d,M[i+12],7,1804603682);d=ff(d,a,bv,c,M[i+13],12,-40341101);c=ff(c,d,a,bv,M[i+14],17,-1502002290);bv=ff(bv,c,d,a,M[i+15],22,1236535329);
    a=gg(a,bv,c,d,M[i+1],5,-165796510);d=gg(d,a,bv,c,M[i+6],9,-1069501632);c=gg(c,d,a,bv,M[i+11],14,643717713);bv=gg(bv,c,d,a,M[i+0],20,-373897302);
    a=gg(a,bv,c,d,M[i+5],5,-701558691);d=gg(d,a,bv,c,M[i+10],9,38016083);c=gg(c,d,a,bv,M[i+15],14,-660478335);bv=gg(bv,c,d,a,M[i+4],20,-405537848);
    a=gg(a,bv,c,d,M[i+9],5,568446438);d=gg(d,a,bv,c,M[i+14],9,-1019803690);c=gg(c,d,a,bv,M[i+3],14,-187363961);bv=gg(bv,c,d,a,M[i+8],20,1163531501);
    a=gg(a,bv,c,d,M[i+13],5,-1444681467);d=gg(d,a,bv,c,M[i+2],9,-51403784);c=gg(c,d,a,bv,M[i+7],14,1735328473);bv=gg(bv,c,d,a,M[i+12],20,-1926607734);
    a=hh(a,bv,c,d,M[i+5],4,-378558);d=hh(d,a,bv,c,M[i+8],11,-2022574463);c=hh(c,d,a,bv,M[i+11],16,1839030562);bv=hh(bv,c,d,a,M[i+14],23,-35309556);
    a=hh(a,bv,c,d,M[i+1],4,-1530992060);d=hh(d,a,bv,c,M[i+4],11,1272893353);c=hh(c,d,a,bv,M[i+7],16,-155497632);bv=hh(bv,c,d,a,M[i+10],23,-1094730640);
    a=hh(a,bv,c,d,M[i+13],4,681279174);d=hh(d,a,bv,c,M[i+0],11,-358537222);c=hh(c,d,a,bv,M[i+3],16,-722521979);bv=hh(bv,c,d,a,M[i+6],23,76029189);
    a=hh(a,bv,c,d,M[i+9],4,-640364487);d=hh(d,a,bv,c,M[i+12],11,-421815835);c=hh(c,d,a,bv,M[i+15],16,530742520);bv=hh(bv,c,d,a,M[i+2],23,-995338651);
    a=ii(a,bv,c,d,M[i+0],6,-198630844);d=ii(d,a,bv,c,M[i+7],10,1126891415);c=ii(c,d,a,bv,M[i+14],15,-1416354905);bv=ii(bv,c,d,a,M[i+5],21,-57434055);
    a=ii(a,bv,c,d,M[i+12],6,1700485571);d=ii(d,a,bv,c,M[i+3],10,-1894986606);c=ii(c,d,a,bv,M[i+10],15,-1051523);bv=ii(bv,c,d,a,M[i+1],21,-2054922799);
    a=ii(a,bv,c,d,M[i+8],6,1873313359);d=ii(d,a,bv,c,M[i+15],10,-30611744);c=ii(c,d,a,bv,M[i+6],15,-1560198380);bv=ii(bv,c,d,a,M[i+13],21,1309151649);
    a=ii(a,bv,c,d,M[i+4],6,-145523070);d=ii(d,a,bv,c,M[i+11],10,-1120210379);c=ii(c,d,a,bv,M[i+2],15,718787259);bv=ii(bv,c,d,a,M[i+9],21,-343485551);
    a=add(a,aa);bv=add(bv,bb);c=add(c,cc);d=add(d,dd);
  }
  return [a,bv,c,d].flatMap(w => [0,1,2,3].map(i => ((w >> (i*8)) & 0xff).toString(16).padStart(2,"0"))).join("");
}

/** Generate a random 8-character alphanumeric salt. */
function randomSalt(): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  return Array.from({ length: 8 }, () =>
    chars[Math.floor(Math.random() * chars.length)]
  ).join("");
}

const FilesWithData: FC = () => {
  const navigate = useNavigate();
  const [refetching, setRefetching] = useState(false);
  const [params] = useSearchParams();
  const path = params.get("q");
  const isRoot = path === null;

  const [plexToken, setPlexToken] = useState<string>(
    () => localStorage.getItem("plex_token") ?? ""
  );
  const [plexPromptServer, setPlexPromptServer] = useState<string | null>(null);
  const tokenInputRef = useRef<HTMLInputElement>(null);

  // Jellyfin auth state
  const [jellyfinPromptServer, setJellyfinPromptServer] = useState<string | null>(null);
  const [jellyfinUsername, setJellyfinUsername] = useState("");
  const [jellyfinPassword, setJellyfinPassword] = useState("");
  const [jellyfinError, setJellyfinError] = useState<string | null>(null);
  const [jellyfinSigningIn, setJellyfinSigningIn] = useState(false);
  const jellyfinUsernameRef = useRef<HTMLInputElement>(null);
  const [jellyfinAuthMode, setJellyfinAuthMode] = useState<"credentials" | "apikey">("credentials");
  const [jellyfinApiKey, setJellyfinApiKey] = useState("");
  const [jellyfinManualEntry, setJellyfinManualEntry] = useState(false);
  const [jellyfinManualUrl, setJellyfinManualUrl] = useState("");
  const jellyfinApiKeyRef = useRef<HTMLInputElement>(null);
  const jellyfinManualUrlRef = useRef<HTMLInputElement>(null);

  // Navidrome auth state
  const [navidromeEntry, setNavidromeEntry] = useState(false);
  const [navidromeUrl, setNavidromeUrl] = useState("");
  const [navidromeUsername, setNavidromeUsername] = useState("");
  const [navidromePassword, setNavidromePassword] = useState("");
  const [navidromeError, setNavidromeError] = useState<string | null>(null);
  const [navidromeConnecting, setNavidromeConnecting] = useState(false);
  const navidromeUrlRef = useRef<HTMLInputElement>(null);

  // Kodi auth state
  const [kodiPromptServer, setKodiPromptServer] = useState<string | null>(null);
  const [kodiUrl, setKodiUrl] = useState("");
  const [kodiUsername, setKodiUsername] = useState("");
  const [kodiPassword, setKodiPassword] = useState("");
  const [kodiError, setKodiError] = useState<string | null>(null);
  const kodiUrlRef = useRef<HTMLInputElement>(null);

  // Resolve the actual path to query: __local__ means music root (no path arg).
  const queryPath = isRoot ? undefined : path === "__local__" ? undefined : path;
  const shouldFetch = !isRoot;

  // Eagerly prefetch UPnP, Plex, and Jellyfin device lists on page load.
  useGetEntriesQuery(
    { path: "upnp://" },
    { staleTime: 0, refetchOnMount: false }
  );
  useGetEntriesQuery(
    { path: "plex://" },
    { staleTime: 0, refetchOnMount: false }
  );
  useGetEntriesQuery(
    { path: "jellyfin://" },
    { staleTime: 0, refetchOnMount: false }
  );
  useGetEntriesQuery(
    { path: "navidrome://" },
    { staleTime: 0, refetchOnMount: false }
  );
  useGetEntriesQuery(
    { path: "kodi://" },
    { staleTime: 0, refetchOnMount: false }
  );

  const { data, isLoading, refetch } = useGetEntriesQuery(
    shouldFetch ? (queryPath !== undefined ? { path: queryPath } : {}) : undefined,
    { enabled: shouldFetch }
  );
  const { mutate: playDirectory } = usePlayDirectoryMutation();

  const rawFiles: File[] = isRoot
    ? ROOT_ENTRIES
    : data?.treeGetEntries.map((x) => ({
        name: x.displayName ?? x.name.split("/").pop()!,
        isDirectory: x.attr === 16,
        path: x.name,
      })) ?? [];

  const files: File[] =
    path === "jellyfin://"
      ? [...rawFiles, JELLYFIN_ADD_MANUALLY]
      : rawFiles;

  const canGoBack = !isRoot;
  const onGoBack = () => navigate(-1);

  const onPlayDirectory = (path: string) => {
    if (
      path.startsWith("upnp://") ||
      path.startsWith("plex://") ||
      path.startsWith("jellyfin://") ||
      path.startsWith("navidrome://") ||
      path.startsWith("kodi://") ||
      path === "__local__"
    )
      return;
    playDirectory({ path, recurse: true });
  };

  const onPlayTrack = (path: string, position: number) => {
    if (
      path.startsWith("upnp://") ||
      path.startsWith("plex://") ||
      path.startsWith("jellyfin://") ||
      path.startsWith("navidrome://") ||
      path.startsWith("kodi://")
    )
      return;
    playDirectory({ path, position });
  };

  const onNavigateDirectory = (file: File) => {
    if (file.path === "__local__") {
      navigate("/files?q=__local__");
    } else if (file.path === "jellyfin://+manual") {
      setJellyfinManualEntry(true);
      setJellyfinManualUrl("");
    } else if (file.path.startsWith("plex://") && isPlexServerPath(file.path)) {
      // Prompt for token before browsing a Plex server.
      setPlexPromptServer(file.path);
    } else if (file.path.startsWith("jellyfin://") && isJellyfinServerPath(file.path)) {
      // Prompt for credentials before browsing a Jellyfin server.
      setJellyfinPromptServer(file.path);
      setJellyfinUsername("");
      setJellyfinPassword("");
      setJellyfinError(null);
    } else if (file.path === "navidrome://") {
      setNavidromeEntry(true);
      setNavidromeUrl("");
      setNavidromeUsername("");
      setNavidromePassword("");
      setNavidromeError(null);
    } else if (file.path.startsWith("navidrome://")) {
      navigate(`/files?q=${encodeURIComponent(file.path)}`);
    } else if (file.path === "kodi://") {
      navigate(`/files?q=${encodeURIComponent(file.path)}`);
    } else if (file.path.startsWith("kodi://") && isKodiServerPath(file.path)) {
      // Prompt for credentials before browsing a Kodi server.
      const rawBase = decodeURIComponent(file.path.slice("kodi://".length)).split("?")[0];
      setKodiUrl(rawBase);
      setKodiUsername("");
      setKodiPassword("");
      setKodiError(null);
      setKodiPromptServer(file.path);
    } else if (file.path.startsWith("kodi://")) {
      navigate(`/files?q=${encodeURIComponent(file.path)}`);
    } else if (
      file.path.startsWith("upnp://") ||
      file.path.startsWith("plex://") ||
      file.path.startsWith("jellyfin://")
    ) {
      navigate(`/files?q=${encodeURIComponent(file.path)}`);
    } else {
      navigate(`/files?q=${file.path}`);
    }
  };

  const handlePlexConnect = () => {
    if (!plexPromptServer) return;
    const navPath = plexToken
      ? `${plexPromptServer}%3FX-Plex-Token%3D${plexToken}`
      : plexPromptServer;
    navigate(`/files?q=${encodeURIComponent(navPath)}`);
    setPlexPromptServer(null);
  };

  const handleJellyfinSignIn = async () => {
    if (!jellyfinPromptServer) return;
    // Decode the base URL from the jellyfin:// path
    const encoded = jellyfinPromptServer.slice("jellyfin://".length);
    const baseUrl = decodeURIComponent(encoded);

    setJellyfinSigningIn(true);
    setJellyfinError(null);
    try {
      const resp = await fetch(`${baseUrl.replace(/\/$/, "")}/Users/AuthenticateByName`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Emby-Authorization":
            'MediaBrowser Client="Rockbox", Device="Browser", DeviceId="rockbox-web", Version="1.0"',
        },
        body: JSON.stringify({ Username: jellyfinUsername, Pw: jellyfinPassword }),
      });
      if (resp.ok) {
        const data = await resp.json();
        const token: string = data.AccessToken;
        const userId: string = data.User.Id;
        // Embed token + userId into the path as query params (percent-encoded)
        const navPath = `${jellyfinPromptServer}%3FX-Jellyfin-Token%3D${encodeURIComponent(
          token
        )}%26userId%3D${encodeURIComponent(userId)}`;
        navigate(`/files?q=${encodeURIComponent(navPath)}`);
        setJellyfinPromptServer(null);
      } else {
        setJellyfinError("Authentication failed. Check username/password.");
      }
    } catch {
      setJellyfinError("Could not reach the Jellyfin server.");
    } finally {
      setJellyfinSigningIn(false);
    }
  };

  const handleJellyfinApiKey = async () => {
    if (!jellyfinPromptServer) return;
    const encoded = jellyfinPromptServer.slice("jellyfin://".length);
    const baseUrl = decodeURIComponent(encoded);

    setJellyfinSigningIn(true);
    setJellyfinError(null);
    try {
      const resp = await fetch(`${baseUrl.replace(/\/$/, "")}/Users`, {
        headers: { "X-Emby-Token": jellyfinApiKey },
      });
      if (resp.ok) {
        const users = await resp.json();
        const userId: string = users[0]?.Id;
        if (!userId) {
          setJellyfinError("No users found for this API key.");
          return;
        }
        const navPath = `${jellyfinPromptServer}%3FX-Jellyfin-Token%3D${encodeURIComponent(
          jellyfinApiKey
        )}%26userId%3D${encodeURIComponent(userId)}`;
        navigate(`/files?q=${encodeURIComponent(navPath)}`);
        setJellyfinPromptServer(null);
      } else {
        setJellyfinError("Invalid API key or insufficient permissions.");
      }
    } catch {
      setJellyfinError("Could not reach the Jellyfin server.");
    } finally {
      setJellyfinSigningIn(false);
    }
  };

  const handleJellyfinManualConnect = () => {
    const url = jellyfinManualUrl.trim();
    if (!url) return;
    const encoded = encodeURIComponent(url);
    const serverPath = `jellyfin://${encoded}`;
    setJellyfinManualEntry(false);
    setJellyfinManualUrl("");
    setJellyfinPromptServer(serverPath);
    setJellyfinUsername("");
    setJellyfinPassword("");
    setJellyfinError(null);
    setJellyfinAuthMode("credentials");
  };

  const handleNavidromeConnect = async () => {
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
      const encoded = encodeURIComponent(credUrl);
      setNavidromeEntry(false);
      navigate(`/files?q=${encodeURIComponent(`navidrome://${encoded}`)}`);
    } catch {
      setNavidromeError("Could not reach the Navidrome server.");
    } finally {
      setNavidromeConnecting(false);
    }
  };

  const handleKodiConnect = () => {
    const baseUrl = kodiUrl.trim().replace(/\/$/, "");
    if (!baseUrl) return;
    const credSuffix = kodiUsername.trim()
      ? `?kodi_user=${encodeURIComponent(kodiUsername)}&kodi_pass=${encodeURIComponent(kodiPassword)}`
      : "";
    const navPath = `kodi://${encodeURIComponent(`${baseUrl}${credSuffix}`)}`;
    setKodiPromptServer(null);
    navigate(`/files?q=${encodeURIComponent(navPath)}`);
  };

  useEffect(() => {
    if (!shouldFetch) return;
    setRefetching(true);
    refetch()
      .then(() => setRefetching(false))
      .catch(() => setRefetching(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path]);

  // Focus token input when the Plex prompt appears.
  useEffect(() => {
    if (plexPromptServer) {
      setTimeout(() => tokenInputRef.current?.focus(), 50);
    }
  }, [plexPromptServer]);

  // Focus username/apikey input when the Jellyfin prompt appears or auth mode changes.
  useEffect(() => {
    if (jellyfinPromptServer) {
      setTimeout(() => {
        if (jellyfinAuthMode === "credentials") {
          jellyfinUsernameRef.current?.focus();
        } else {
          jellyfinApiKeyRef.current?.focus();
        }
      }, 50);
    }
  }, [jellyfinPromptServer, jellyfinAuthMode]);

  // Focus manual URL input when the manual entry prompt appears.
  useEffect(() => {
    if (jellyfinManualEntry) {
      setTimeout(() => jellyfinManualUrlRef.current?.focus(), 50);
    }
  }, [jellyfinManualEntry]);

  // Focus Navidrome URL input when the entry form appears.
  useEffect(() => {
    if (navidromeEntry) {
      setTimeout(() => navidromeUrlRef.current?.focus(), 50);
    }
  }, [navidromeEntry]);

  // Focus Kodi URL input when the prompt appears.
  useEffect(() => {
    if (kodiPromptServer) {
      setTimeout(() => kodiUrlRef.current?.focus(), 50);
    }
  }, [kodiPromptServer]);

  return (
    <>
      <Files
        files={files}
        canGoBack={canGoBack}
        onGoBack={onGoBack}
        refetching={shouldFetch && (isLoading || refetching)}
        onPlayDirectory={onPlayDirectory}
        onPlayTrack={onPlayTrack}
        onNavigateDirectory={onNavigateDirectory}
      />

      {plexPromptServer && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => setPlexPromptServer(null)}
        >
          <div
            className="bg-[var(--theme-bg,#1a1a1a)] rounded-xl p-6 w-[320px] shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <p className="text-[var(--theme-text)] font-medium text-sm mb-1">
              Plex Token
            </p>
            <p className="text-[#888] text-xs mb-4">
              Required for private servers. Leave blank for public ones.
            </p>
            <input
              ref={tokenInputRef}
              type="password"
              placeholder="paste your X-Plex-Token here"
              value={plexToken}
              onChange={(e) => {
                const t = e.target.value;
                setPlexToken(t);
                localStorage.setItem("plex_token", t);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") handlePlexConnect();
                if (e.key === "Escape") setPlexPromptServer(null);
              }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-5"
            />
            <div className="flex gap-2 justify-end">
              <button
                className="px-4 py-1.5 rounded-lg text-sm text-[#888] hover:bg-white/5 cursor-pointer border-0 bg-transparent"
                onClick={() => setPlexPromptServer(null)}
              >
                Cancel
              </button>
              <button
                className="px-4 py-1.5 rounded-lg text-sm bg-[var(--theme-primary,#e5a00d)] text-black font-medium cursor-pointer border-0 hover:opacity-90"
                onClick={handlePlexConnect}
              >
                Connect
              </button>
            </div>
          </div>
        </div>
      )}

      {jellyfinPromptServer && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => {
            if (!jellyfinSigningIn) setJellyfinPromptServer(null);
          }}
        >
          <div
            className="bg-[var(--theme-bg,#1a1a1a)] rounded-xl p-6 w-[320px] shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <p className="text-[var(--theme-text)] font-medium text-sm mb-4">
              Sign in to Jellyfin
            </p>
            {/* Auth mode tabs */}
            <div className="flex gap-1 bg-black/20 rounded-lg p-1 mb-4">
              <button
                className={`flex-1 text-xs py-1 rounded-md cursor-pointer border-0 ${
                  jellyfinAuthMode === "credentials"
                    ? "bg-[var(--theme-primary,#e5a00d)] text-black font-medium"
                    : "bg-transparent text-[#888]"
                }`}
                onClick={() => { setJellyfinAuthMode("credentials"); setJellyfinError(null); }}
              >
                Credentials
              </button>
              <button
                className={`flex-1 text-xs py-1 rounded-md cursor-pointer border-0 ${
                  jellyfinAuthMode === "apikey"
                    ? "bg-[var(--theme-primary,#e5a00d)] text-black font-medium"
                    : "bg-transparent text-[#888]"
                }`}
                onClick={() => { setJellyfinAuthMode("apikey"); setJellyfinError(null); }}
              >
                API Key
              </button>
            </div>
            {jellyfinAuthMode === "credentials" ? (
              <>
                <input
                  ref={jellyfinUsernameRef}
                  type="text"
                  placeholder="Username"
                  value={jellyfinUsername}
                  onChange={(e) => setJellyfinUsername(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleJellyfinSignIn();
                    if (e.key === "Escape" && !jellyfinSigningIn) setJellyfinPromptServer(null);
                  }}
                  className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-3"
                />
                <input
                  type="password"
                  placeholder="Password"
                  value={jellyfinPassword}
                  onChange={(e) => setJellyfinPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleJellyfinSignIn();
                    if (e.key === "Escape" && !jellyfinSigningIn) setJellyfinPromptServer(null);
                  }}
                  className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-4"
                />
              </>
            ) : (
              <input
                ref={jellyfinApiKeyRef}
                type="password"
                placeholder="Paste your API key"
                value={jellyfinApiKey}
                onChange={(e) => setJellyfinApiKey(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleJellyfinApiKey();
                  if (e.key === "Escape" && !jellyfinSigningIn) setJellyfinPromptServer(null);
                }}
                className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-4"
              />
            )}
            {jellyfinError && (
              <p className="text-red-400 text-xs mb-3">{jellyfinError}</p>
            )}
            <div className="flex gap-2 justify-end">
              <button
                className="px-4 py-1.5 rounded-lg text-sm text-[#888] hover:bg-white/5 cursor-pointer border-0 bg-transparent disabled:opacity-40"
                disabled={jellyfinSigningIn}
                onClick={() => setJellyfinPromptServer(null)}
              >
                Cancel
              </button>
              <button
                className="px-4 py-1.5 rounded-lg text-sm bg-[var(--theme-primary,#e5a00d)] text-black font-medium cursor-pointer border-0 hover:opacity-90 disabled:opacity-40"
                disabled={jellyfinSigningIn}
                onClick={jellyfinAuthMode === "credentials" ? handleJellyfinSignIn : handleJellyfinApiKey}
              >
                {jellyfinSigningIn ? "Signing in…" : "Sign in"}
              </button>
            </div>
          </div>
        </div>
      )}

      {jellyfinManualEntry && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => setJellyfinManualEntry(false)}
        >
          <div
            className="bg-[var(--theme-bg,#1a1a1a)] rounded-xl p-6 w-[320px] shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <p className="text-[var(--theme-text)] font-medium text-sm mb-1">
              Add Jellyfin Server
            </p>
            <p className="text-[#888] text-xs mb-4">
              Enter the server URL (e.g. http://192.168.1.10:8096)
            </p>
            <input
              ref={jellyfinManualUrlRef}
              type="text"
              placeholder="http://192.168.1.x:8096"
              value={jellyfinManualUrl}
              onChange={(e) => setJellyfinManualUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleJellyfinManualConnect();
                if (e.key === "Escape") setJellyfinManualEntry(false);
              }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-5"
            />
            <div className="flex gap-2 justify-end">
              <button
                className="px-4 py-1.5 rounded-lg text-sm text-[#888] hover:bg-white/5 cursor-pointer border-0 bg-transparent"
                onClick={() => setJellyfinManualEntry(false)}
              >
                Cancel
              </button>
              <button
                className="px-4 py-1.5 rounded-lg text-sm bg-[var(--theme-primary,#e5a00d)] text-black font-medium cursor-pointer border-0 hover:opacity-90"
                onClick={handleJellyfinManualConnect}
              >
                Connect
              </button>
            </div>
          </div>
        </div>
      )}

      {kodiPromptServer && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => setKodiPromptServer(null)}
        >
          <div
            className="bg-[var(--theme-bg,#1a1a1a)] rounded-xl p-6 w-[320px] shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <p className="text-[var(--theme-text)] font-medium text-sm mb-1">Connect to Kodi</p>
            <p className="text-[#888] text-xs mb-4">Username and password are optional for local servers.</p>
            <input
              ref={kodiUrlRef}
              type="text"
              placeholder="http://192.168.1.x:8080"
              value={kodiUrl}
              onChange={(e) => setKodiUrl(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleKodiConnect(); if (e.key === "Escape") setKodiPromptServer(null); }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-3"
            />
            <input
              type="text"
              placeholder="Username (optional)"
              value={kodiUsername}
              onChange={(e) => setKodiUsername(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleKodiConnect(); if (e.key === "Escape") setKodiPromptServer(null); }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-3"
            />
            <input
              type="password"
              placeholder="Password (optional)"
              value={kodiPassword}
              onChange={(e) => setKodiPassword(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleKodiConnect(); if (e.key === "Escape") setKodiPromptServer(null); }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-4"
            />
            {kodiError && <p className="text-red-400 text-xs mb-3">{kodiError}</p>}
            <div className="flex gap-2 justify-end">
              <button
                className="px-4 py-1.5 rounded-lg text-sm text-[#888] hover:bg-white/5 cursor-pointer border-0 bg-transparent"
                onClick={() => setKodiPromptServer(null)}
              >
                Cancel
              </button>
              <button
                className="px-4 py-1.5 rounded-lg text-sm bg-[var(--theme-primary,#e5a00d)] text-black font-medium cursor-pointer border-0 hover:opacity-90"
                onClick={handleKodiConnect}
              >
                Connect
              </button>
            </div>
          </div>
        </div>
      )}

      {navidromeEntry && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => { if (!navidromeConnecting) setNavidromeEntry(false); }}
        >
          <div
            className="bg-[var(--theme-bg,#1a1a1a)] rounded-xl p-6 w-[320px] shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <p className="text-[var(--theme-text)] font-medium text-sm mb-1">
              Connect to Navidrome
            </p>
            <p className="text-[#888] text-xs mb-4">
              Enter your Navidrome server URL and credentials.
            </p>
            <input
              ref={navidromeUrlRef}
              type="text"
              placeholder="http://192.168.1.x:4533"
              value={navidromeUrl}
              onChange={(e) => setNavidromeUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleNavidromeConnect();
                if (e.key === "Escape" && !navidromeConnecting) setNavidromeEntry(false);
              }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-3"
            />
            <input
              type="text"
              placeholder="Username"
              value={navidromeUsername}
              onChange={(e) => setNavidromeUsername(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleNavidromeConnect();
                if (e.key === "Escape" && !navidromeConnecting) setNavidromeEntry(false);
              }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-3"
            />
            <input
              type="password"
              placeholder="Password"
              value={navidromePassword}
              onChange={(e) => setNavidromePassword(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleNavidromeConnect();
                if (e.key === "Escape" && !navidromeConnecting) setNavidromeEntry(false);
              }}
              className="w-full bg-[var(--theme-input-bg,#2a2a2a)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2 border border-[var(--theme-border,#444)] outline-none focus:border-[var(--theme-primary,#e5a00d)] mb-4"
            />
            {navidromeError && (
              <p className="text-red-400 text-xs mb-3">{navidromeError}</p>
            )}
            <div className="flex gap-2 justify-end">
              <button
                className="px-4 py-1.5 rounded-lg text-sm text-[#888] hover:bg-white/5 cursor-pointer border-0 bg-transparent disabled:opacity-40"
                disabled={navidromeConnecting}
                onClick={() => setNavidromeEntry(false)}
              >
                Cancel
              </button>
              <button
                className="px-4 py-1.5 rounded-lg text-sm bg-[var(--theme-primary,#e5a00d)] text-black font-medium cursor-pointer border-0 hover:opacity-90 disabled:opacity-40"
                disabled={navidromeConnecting}
                onClick={handleNavidromeConnect}
              >
                {navidromeConnecting ? "Connecting…" : "Connect"}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default FilesWithData;
