import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const TrackRowSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width="100%"
      height={48}
      viewBox="0 0 1000 48"
      preserveAspectRatio="none"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <rect x="8" y="17" rx="2" ry="2" width="22" height="14" />
      <rect x="44" y="4" rx="3" ry="3" width="40" height="40" />
      <rect x="95" y="9" rx="2" ry="2" width="200" height="14" />
      <rect x="95" y="28" rx="2" ry="2" width="130" height="10" />
      <rect x="360" y="9" rx="2" ry="2" width="180" height="14" />
      <rect x="600" y="9" rx="2" ry="2" width="170" height="14" />
      <rect x="840" y="9" rx="2" ry="2" width="60" height="14" />
    </ContentLoader>
  );
};

export default TrackRowSkeleton;
