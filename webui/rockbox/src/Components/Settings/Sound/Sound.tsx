import { FC, useEffect, useState } from "react";
import styles, { Item, Section, SettingsTitle } from "./styles";
import { Slider } from "@mui/material";
import Equalizer from "./Equalizer";

export type SoundProps = {
  bass: number;
  treble: number;
  balance: number;
  onBassChange: (bass: number) => void;
  onTrebleChange: (treble: number) => void;
  onBalanceChange: (balance: number) => void;
};

const Sound: FC<SoundProps> = (props) => {
  const [bass, setBass] = useState(props.bass);
  const [treble, setTreble] = useState(props.treble);
  const [balance, setBalance] = useState(props.balance);

  useEffect(() => {
    setBass(props.bass);
    setTreble(props.treble);
    setBalance(props.balance);
  }, [props.bass, props.treble, props.balance]);

  const computeCutOff = (cutoff: number) => {
    // -24 dB to 24 dB
    return ((cutoff + 24) / 48) * 100;
  };

  const handleBassChange = (value: number) => {
    setBass(Math.floor((value / 100) * 48 - 24));
  };

  const handleTrebleChange = (value: number) => {
    setTreble(Math.floor((value / 100) * 48 - 24));
  };

  const handleBalanceChange = (value: number) => {
    setBalance(value);
  };

  const onBassChangeCommitted = (value: number) => {
    props.onBassChange(Math.floor((value / 100) * 48 - 24));
  };

  const onTrebleChangeCommitted = (value: number) => {
    props.onTrebleChange(Math.floor((value / 100) * 48 - 24));
  };

  const onBalanceChangeCommitted = (value: number) => {
    props.onBalanceChange(value);
  };

  return (
    <>
      <SettingsTitle>Sound</SettingsTitle>
      <Section>
        <Item>
          <div>Bass</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(bass)}
              onChange={(_event, value) => handleBassChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onBassChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) =>
                `${Math.floor((value / 100) * 48 - 24)} dB`
              }
              min={0}
              max={100}
              step={1}
            />
          </div>
        </Item>
        <Item>
          <div>Treble</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(treble)}
              onChange={(_event, value) => handleTrebleChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onTrebleChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) =>
                `${Math.floor((value / 100) * 48 - 24)} dB`
              }
              min={0}
              max={100}
              step={1}
            />
          </div>
        </Item>
        <Item>
          <div>Balance</div>
          <div style={{ width: 120 }}>
            <Slider
              value={balance}
              onChange={(_event, value) => handleBalanceChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onBalanceChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) => `${value} %`}
              min={-100}
              max={100}
              step={1}
            />
          </div>
        </Item>
        <Equalizer />
      </Section>
    </>
  );
};

export default Sound;
