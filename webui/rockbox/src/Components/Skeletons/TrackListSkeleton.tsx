import { FC } from "react";
import TrackRowSkeleton from "./TrackRowSkeleton";

type Props = { rows?: number };

const TrackListSkeleton: FC<Props> = ({ rows = 10 }) => (
  <div>
    {Array.from({ length: rows }).map((_, i) => (
      <TrackRowSkeleton key={i} />
    ))}
  </div>
);

export default TrackListSkeleton;
