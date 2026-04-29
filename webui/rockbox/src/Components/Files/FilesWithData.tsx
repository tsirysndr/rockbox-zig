import { FC, useEffect, useState } from "react";
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
];

const FilesWithData: FC = () => {
  const navigate = useNavigate();
  const [refetching, setRefetching] = useState(false);
  const [params] = useSearchParams();
  const path = params.get("q");
  const isRoot = path === null;

  // Resolve the actual path to query: __local__ means music root (no path arg).
  const queryPath = isRoot ? undefined : path === "__local__" ? undefined : path;
  const shouldFetch = !isRoot;

  // Eagerly prefetch UPnP devices on page load so the list appears instantly when
  // the user opens "UPnP Devices". React Query caches the result; on navigation it
  // serves the stale data immediately and revalidates in the background.
  useGetEntriesQuery(
    { path: "upnp://" },
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
    if (path.startsWith("upnp://") || path === "__local__") return;
    playDirectory({ path, recurse: true });
  };

  const onPlayTrack = (path: string, position: number) => {
    if (path.startsWith("upnp://")) return;
    playDirectory({ path, position });
  };

  const onNavigateDirectory = (file: File) => {
    if (file.path === "__local__") {
      navigate("/files?q=__local__");
    } else if (file.path.startsWith("upnp://")) {
      navigate(`/files?q=${encodeURIComponent(file.path)}`);
    } else {
      navigate(`/files?q=${file.path}`);
    }
  };

  useEffect(() => {
    if (!shouldFetch) return;
    setRefetching(true);
    refetch()
      .then(() => setRefetching(false))
      .catch(() => setRefetching(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path]);

  return (
    <Files
      files={files}
      canGoBack={canGoBack}
      onGoBack={onGoBack}
      refetching={shouldFetch && (isLoading || refetching)}
      onPlayDirectory={onPlayDirectory}
      onPlayTrack={onPlayTrack}
      onNavigateDirectory={onNavigateDirectory}
    />
  );
};

export default FilesWithData;
