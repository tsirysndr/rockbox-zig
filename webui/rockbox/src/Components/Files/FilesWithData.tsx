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
  const { data, refetch } = useGetEntriesQuery();
  const [params] = useSearchParams();
  const path = params.get("q");
  const canGoBack = !!path;
  const [playDirectory] = usePlayDirectoryMutation();

  const files =
    data?.treeGetEntries.map((x) => ({
      name: x.name.split("/").pop()!,
      isDirectory: x.attr === 16,
      path: x.name,
    })) || [];

  const onGoBack = () => navigate(-1);

  const onPlayDirectory = (path: string) => {
    playDirectory({
      variables: {
        path,
        recurse: true,
      },
    });
  };

  const onPlayTrack = (path: string, position: number) => {
    playDirectory({
      variables: {
        path,
        position,
      },
    });
  };

  useEffect(() => {
    setRefetching(true);
    refetch({
      path,
    })
      .then(() => setRefetching(false))
      .catch(() => setRefetching(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path]);

  return (
    <Files
      files={files}
      canGoBack={canGoBack}
      onGoBack={onGoBack}
      refetching={refetching}
      onPlayDirectory={onPlayDirectory}
      onPlayTrack={onPlayTrack}
    />
  );
};

export default FilesWithData;
