import { FC, SyntheticEvent, useEffect, useState } from "react";
import Slider from "@mui/material/Slider";
import styles, { Container } from "./styles";

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
    <Container>
      <div style={{ width: 100, marginTop: 5 }}>
        <Slider
          aria-label="Volume"
          value={volume}
          onChange={(_event, value) => setVolume(value as number)}
          onChangeCommitted={handleVolumeChange}
          sx={styles.slider}
        />
      </div>
    </Container>
  );
};

export default Volume;
