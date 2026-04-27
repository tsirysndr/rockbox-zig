import { FC, SyntheticEvent, useEffect, useState } from "react";
import { useTheme } from "@emotion/react";
import Slider from "@mui/material/Slider";
import { Container } from "./styles";

export type VolumeProps = {
  volume: number;
  onVolumeChange: (volume: number) => Promise<void>;
};

const Volume: FC<VolumeProps> = (props) => {
  const theme = useTheme();
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
    <Container>
      <div style={{ width: 100, marginTop: 5 }}>
        <Slider
          aria-label="Volume"
          value={volume}
          onChange={(_event, value) => setVolume(value as number)}
          onChangeCommitted={handleVolumeChange}
          sx={{
            color: theme.colors.primary,
            "& .MuiSlider-track": { border: "none" },
            "& .MuiSlider-thumb": {
              width: 18,
              height: 18,
              backgroundColor: theme.colors.text,
              "&::before": { boxShadow: "0 4px 8px rgba(0,0,0,0.18)" },
              "&:hover, &.Mui-focusVisible, &.Mui-active": { boxShadow: "none" },
            },
          }}
        />
      </div>
    </Container>
  );
};

export default Volume;
