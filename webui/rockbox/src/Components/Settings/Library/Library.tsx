import { FC, useEffect, useState } from "react";
import { Item, Section, SettingsTitle } from "./styles";
import { Input } from "baseui/input";

export type LibraryProps = {
  musicDir: string;
  onSaveMusicDirectoryPath: (musicDir: string) => void;
};

const Library: FC<LibraryProps> = (props) => {
  const [musicFolder, setMusicFolder] = useState<string>(props.musicDir);

  useEffect(() => {
    setMusicFolder(props.musicDir);
  }, [props.musicDir]);

  return (
    <>
      <SettingsTitle>Library</SettingsTitle>
      <Section>
        <Item style={{ height: 80 }}>
          <div>Load music from folder</div>
          <div>
            <Input
              value={musicFolder}
              onChange={(e) => {
                setMusicFolder(e.target.value);
                props.onSaveMusicDirectoryPath(e.target.value);
              }}
              placeholder="Music folder"
            />
          </div>
        </Item>
      </Section>
    </>
  );
};

export default Library;
