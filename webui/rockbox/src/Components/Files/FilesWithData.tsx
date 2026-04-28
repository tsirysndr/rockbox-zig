import { FC, useEffect, useState } from "react";
import Files from "./Files";
import {
  useGetEntriesQuery,
  usePlayDirectoryMutation,
} from "../../Hooks/GraphQL";
import { useNavigate, useSearchParams } from "react-router-dom";

const FilesWithData: FC = () => {
  const navigate = useNavigate();
  const [refetching, setRefetching] = useState(false);
  const [params] = useSearchParams();
  const path = params.get("q");
  const { data, isLoading, refetch } = useGetEntriesQuery(path !== null ? { path } : undefined);
  const canGoBack = !!path;
  const { mutate: playDirectory } = usePlayDirectoryMutation();

  const files =
    data?.treeGetEntries.map((x) => ({
      name: x.name.split("/").pop()!,
      isDirectory: x.attr === 16,
      path: x.name,
    })) || [];

  const onGoBack = () => navigate(-1);

  const onPlayDirectory = (path: string) => {
    playDirectory({ path, recurse: true });
  };

  const onPlayTrack = (path: string, position: number) => {
    playDirectory({ path, position });
  };

  useEffect(() => {
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
      refetching={isLoading || refetching}
      onPlayDirectory={onPlayDirectory}
      onPlayTrack={onPlayTrack}
    />
  );
};

export default FilesWithData;
