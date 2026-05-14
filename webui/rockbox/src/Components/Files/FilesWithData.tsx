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
];

/** True when a plex:// path refers to a server entry (no unencoded `/` after the scheme). */
function isPlexServerPath(path: string): boolean {
  const rest = path.slice("plex://".length);
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

  // Resolve the actual path to query: __local__ means music root (no path arg).
  const queryPath = isRoot ? undefined : path === "__local__" ? undefined : path;
  const shouldFetch = !isRoot;

  // Eagerly prefetch UPnP and Plex device lists on page load.
  useGetEntriesQuery(
    { path: "upnp://" },
    { staleTime: 0, refetchOnMount: false }
  );
  useGetEntriesQuery(
    { path: "plex://" },
    { staleTime: 0, refetchOnMount: false }
  );

  const { data, isLoading, refetch } = useGetEntriesQuery(
    shouldFetch ? (queryPath !== undefined ? { path: queryPath } : {}) : undefined,
    { enabled: shouldFetch }
  );
  const { mutate: playDirectory } = usePlayDirectoryMutation();

  const files: File[] = isRoot
    ? ROOT_ENTRIES
    : data?.treeGetEntries.map((x) => ({
        name: x.displayName ?? x.name.split("/").pop()!,
        isDirectory: x.attr === 16,
        path: x.name,
      })) ?? [];

  const canGoBack = !isRoot;
  const onGoBack = () => navigate(-1);

  const onPlayDirectory = (path: string) => {
    if (path.startsWith("upnp://") || path.startsWith("plex://") || path === "__local__") return;
    playDirectory({ path, recurse: true });
  };

  const onPlayTrack = (path: string, position: number) => {
    if (path.startsWith("upnp://") || path.startsWith("plex://")) return;
    playDirectory({ path, position });
  };

  const onNavigateDirectory = (file: File) => {
    if (file.path === "__local__") {
      navigate("/files?q=__local__");
    } else if (file.path.startsWith("plex://") && isPlexServerPath(file.path)) {
      // Prompt for token before browsing a Plex server.
      setPlexPromptServer(file.path);
    } else if (file.path.startsWith("upnp://") || file.path.startsWith("plex://")) {
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

  useEffect(() => {
    if (!shouldFetch) return;
    setRefetching(true);
    refetch()
      .then(() => setRefetching(false))
      .catch(() => setRefetching(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path]);

  // Focus token input when the prompt appears.
  useEffect(() => {
    if (plexPromptServer) {
      setTimeout(() => tokenInputRef.current?.focus(), 50);
    }
  }, [plexPromptServer]);

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
    </>
  );
};

export default FilesWithData;
