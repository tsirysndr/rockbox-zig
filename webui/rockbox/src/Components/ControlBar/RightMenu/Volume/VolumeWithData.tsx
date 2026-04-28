import {
  useAdjustVolumeMutation,
  useGetVolumeQuery,
} from "../../../../Hooks/GraphQL";
import Volume from "./Volume";
import { FC, useMemo } from "react";

const VolumeWithData: FC = () => {
  const { data, refetch } = useGetVolumeQuery();
  const { mutateAsync: adjustVolumeAsync } = useAdjustVolumeMutation();

  const min = data?.volume.min ?? -80;
  const max = data?.volume.max ?? 0;
  const currentDb = data?.volume.volume ?? min;

  // Map dB value to 0–100 range for the slider
  const range = max - min;
  const volume = useMemo(() => {
    return range > 0 ? Math.round(((currentDb - min) / range) * 100) : 0;
  }, [currentDb, min, max]);

  const onVolumeChange = async (newVolume: number) => {
    // newVolume is 0–100; convert to dB then compute steps
    const newDb = range > 0 ? Math.round((newVolume / 100) * range + min) : min;
    const steps = newDb - currentDb;
    if (steps !== 0) {
      await adjustVolumeAsync({ steps });
      await refetch();
    }
  };

  return <Volume volume={volume} onVolumeChange={onVolumeChange} />;
};

export default VolumeWithData;
