import { FC, useEffect, useState } from "react";
import styles, { Container, Item } from "./styles";
import Switch from "../../../Switch";
import { Slider } from "@mui/material";

export type EqualizerProps = {
  eqEnabled: boolean;
  onEnableEq: (enabled: boolean) => void;
  eqBandSettings: {
    q: number;
    gain: number;
    cutoff: number;
  }[];
  onEqBandSettingsChange: (
    bandSettings: {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  ) => void;
};

const Equalizer: FC<EqualizerProps> = (props) => {
  const [eqEnabled, setEqEnabled] = useState(props.eqEnabled);
  const [eqBandSettings, setEqBandSettings] = useState<
    {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  >(props.eqBandSettings);

  useEffect(() => {
    setEqEnabled(props.eqEnabled);
    setEqBandSettings(props.eqBandSettings);
  }, [props.eqEnabled, props.eqBandSettings]);

  const formatLabel = (value: number) => {
    const labels: Record<number, string> = {
      64: "32 Hz",
      125: "64 Hz",
      250: "125 Hz",
      500: "250 Hz",
      1000: "500 Hz",
      2000: "1 kHz",
      4000: "2 kHz",
      8000: "4 kHz",
      16000: "8 kHz",
      0: "16 kHz",
    };
    return `${labels[value]}`;
  };

  const computeCutOff = (cutoff: number) => {
    // -24 dB to 24 dB
    return ((cutoff + 24) / 48) * 100;
  };

  const handleChange = (value: number, index: number) => {
    const newBandSettings = [...eqBandSettings];
    newBandSettings[index] = {
      ...newBandSettings[index],
      cutoff: Math.floor(((value / 100) * 48 - 24) * 10),
    };
    setEqBandSettings(newBandSettings);
  };

  const onChangeCommitted = (value: number, index: number) => {
    const newBandSettings = [...eqBandSettings];
    newBandSettings[index] = {
      ...newBandSettings[index],
      cutoff: Math.floor(((value / 100) * 48 - 24) * 10),
    };
    props.onEqBandSettingsChange(newBandSettings);
  };

  return (
    <>
      <Item>
        <div>Equalizer</div>
        <div>
          <Switch
            checked={eqEnabled}
            onChange={() => {
              props.onEnableEq(!eqEnabled);
              setEqEnabled(!eqEnabled);
            }}
          />
        </div>
      </Item>
      <div>
        <Container>
          {eqBandSettings.map((band, index) => (
            <div key={index}>
              <Slider
                value={computeCutOff(band.cutoff * 0.1)}
                onChange={(_event, value) =>
                  handleChange(value as number, index)
                }
                onChangeCommitted={(_event, value) =>
                  onChangeCommitted(value as number, index)
                }
                sx={styles.sliderIOS}
                valueLabelDisplay="auto"
                orientation="vertical"
                min={0}
                max={100}
                step={0.1}
                valueLabelFormat={(value) =>
                  `${((value / 100) * 48 - 24).toFixed(1)} dB`
                }
              />
              <div>{formatLabel(band.q)}</div>
            </div>
          ))}
        </Container>
      </div>
    </>
  );
};

export default Equalizer;
