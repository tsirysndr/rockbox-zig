import {
  useAdjustVolumeMutation,
  useGetGlobalSettingsQuery,
} from "../../../../Hooks/GraphQL";
import Volume from "./Volume";
import { FC, useMemo } from "react";

const VolumeWithData: FC = () => {
  const { data, refetch } = useGetGlobalSettingsQuery();
  const [adjustVolume] = useAdjustVolumeMutation();
  const volume = useMemo(() => {
    return Math.min((data?.globalSettings.volume || 0) + 80, 80);
  }, [data]);

  const onVolumeChange = async (newVolume: number) => {
    const steps = Math.min(newVolume, 80) - Math.min(volume, 80);
    await adjustVolume({
      variables: {
        steps,
      },
    });
    await refetch();
  };

  return <Volume volume={volume} onVolumeChange={onVolumeChange} />;
};

export default VolumeWithData;
