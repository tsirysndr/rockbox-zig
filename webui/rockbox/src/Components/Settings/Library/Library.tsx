import { FC, useState } from "react";
import { Item, Section, SettingsTitle } from "./styles";
import { Input } from "baseui/input";

const Library: FC = () => {
  const [musicFolder, setMusicFolder] = useState<string>("$HOME/Music");
  return (
    <>
      <SettingsTitle>Library</SettingsTitle>
      <Section>
        <Item style={{ height: 80 }}>
          <div>Load music from folder</div>
          <div>
            <Input
              value={musicFolder}
              onChange={(e) => setMusicFolder(e.target.value)}
              placeholder="Music folder"
            />
          </div>
        </Item>
      </Section>
    </>
  );
};

export default Library;
