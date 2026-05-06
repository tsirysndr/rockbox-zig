import { useTimeFormat } from "../../Hooks/useFormat";

const Progress = ({ currentTime, duration }) => {
  const { formatTime } = useTimeFormat();
  const pct = duration > 0 ? (currentTime / duration) * 100 : 0;

  return (
    <div className="px-[calc(5vw-10px)] pt-5">
      <div className="mx-2.5 h-1 rounded-full bg-white/20">
        <div
          className="h-1 rounded-full bg-[rgb(254,9,156)]"
          style={{ width: `${pct}%` }}
        />
      </div>
      <div className="flex justify-between px-2.5">
        <div>{formatTime(currentTime)}</div>
        <div>{formatTime(duration)}</div>
      </div>
    </div>
  );
};

Progress.defaultProps = {
  currentTime: 0,
  duration: 0,
};

export default Progress;
