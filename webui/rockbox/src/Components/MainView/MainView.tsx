import { FC, ReactNode } from "react";

export type MainViewProps = {
  cover?: string;
  children?: ReactNode;
};

const MainView: FC<MainViewProps> = ({ cover, children }) => {
  return (
    <div className={`flex flex-1 flex-col relative w-full md:w-[calc(100%-240px)] bg-[var(--theme-background)] bg-center bg-no-repeat bg-cover${cover ? ` bg-[url(${cover})]` : ''}`}>
      <div className={`h-screen${cover ? ' bg-[rgba(0,0,0,0.7)] backdrop-blur-[30px]' : ''}`}>{children}</div>
    </div>
  );
};

export default MainView;
