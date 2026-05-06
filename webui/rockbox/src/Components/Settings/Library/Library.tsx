import { FC, useEffect, useState } from "react";
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
      <div className="text-base font-semibold mb-4">Library</div>
      <div className="mb-[50px] text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between min-h-[50px]" style={{ height: 80 }}>
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
        </div>
      </div>
    </>
  );
};

export default Library;
