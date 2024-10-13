import { FC, useEffect, useState } from "react";
import Files from "./Files";
import { useGetEntriesQuery } from "../../Hooks/GraphQL";
import { useNavigate, useSearchParams } from "react-router-dom";

const FilesWithData: FC = () => {
  const navigate = useNavigate();
  const [refetching, setRefetching] = useState(false);
  const { data, refetch, loading } = useGetEntriesQuery();
  const [params] = useSearchParams();
  const path = params.get("q");
  const canGoBack = !!path;

  const files =
    data?.treeGetEntries.map((x) => ({
      name: x.name.split("/").pop()!,
      isDirectory: x.attr === 16,
      path: x.name,
    })) || [];

  const onGoBack = () => navigate(-1);

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
    />
  );
};

export default FilesWithData;
