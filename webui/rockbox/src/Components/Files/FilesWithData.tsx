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
      path === "__local__"
    )
      return;
    playDirectory({ path, recurse: true });
  };

  const onPlayTrack = (path: string, position: number) => {
    if (
      path.startsWith("upnp://") ||
      path.startsWith("plex://") ||
      path.startsWith("jellyfin://")
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
    </>
  );
};

export default FilesWithData;
