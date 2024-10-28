import { FC } from "react";
import Library from "./Library";
import {
  useGetGlobalSettingsQuery,
  useSaveSettingsMutation,
} from "../../../Hooks/GraphQL";

const LibraryWithData: FC = () => {
  const { data } = useGetGlobalSettingsQuery();
  const [saveSettings] = useSaveSettingsMutation();

  const onSaveMusicDirectoryPath = (musicDir: string) =>
    saveSettings({
      variables: {
        settings: {
          musicDir,
        },
      },
    });

  return (
    <Library
      musicDir={data?.globalSettings.musicDir || ""}
      onSaveMusicDirectoryPath={onSaveMusicDirectoryPath}
    />
  );
};

export default LibraryWithData;
