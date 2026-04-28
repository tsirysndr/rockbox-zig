import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const DetailHeaderSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width="100%"
      height={260}
      viewBox="0 0 700 260"
      preserveAspectRatio="xMinYMid meet"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <rect x="0" y="10" rx="4" ry="4" width="240" height="240" />
      <rect x="265" y="30" rx="3" ry="3" width="280" height="26" />
      <rect x="265" y="68" rx="3" ry="3" width="190" height="18" />
      <rect x="265" y="100" rx="3" ry="3" width="130" height="14" />
      <rect x="265" y="124" rx="3" ry="3" width="90" height="12" />
      <rect x="265" y="165" rx="6" ry="6" width="110" height="38" />
      <rect x="385" y="165" rx="6" ry="6" width="120" height="38" />
    </ContentLoader>
  );
};

export default DetailHeaderSkeleton;
