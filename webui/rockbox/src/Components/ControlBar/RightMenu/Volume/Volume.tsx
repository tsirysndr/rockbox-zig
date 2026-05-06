import { FC, SyntheticEvent, useEffect, useState } from "react";
import Slider from "@mui/material/Slider";


export type VolumeProps = {
  volume: number;
  onVolumeChange: (volume: number) => Promise<void>;
};

const Volume: FC<VolumeProps> = (props) => {
  const [volume, setVolume] = useState<number>(props.volume);

  useEffect(() => {
    setVolume(props.volume);
  }, [props.volume]);

  const handleVolumeChange = async (
    _event: Event | SyntheticEvent<Element, Event>,
    value: number | number[]
  ) => {
    await props.onVolumeChange(value as number);
  };

  return (
    <div className="flex-1 flex justify-center items-center">
      <div style={{ width: 100, marginTop: 5 }}>
        <Slider
          aria-label="Volume"
          value={volume}
          min={0}
          max={100}
          onChange={(_event, value) => setVolume(value as number)}
          onChangeCommitted={handleVolumeChange}
          sx={{
            color: "var(--theme-primary)",
            "& .MuiSlider-track": { border: "none" },
            "& .MuiSlider-thumb": {
              width: 18,
              height: 18,
              backgroundColor: "var(--theme-text)",
              "&::before": { boxShadow: "0 4px 8px rgba(0,0,0,0.18)" },
              "&:hover, &.Mui-focusVisible, &.Mui-active": { boxShadow: "none" },
            },
          }}
        />
      </div>
    </div>
  );
};

export default Volume;
