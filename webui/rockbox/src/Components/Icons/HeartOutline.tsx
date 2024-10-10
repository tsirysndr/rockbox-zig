import * as React from "react";

export type HeartOutlineProps = {
  size?: number;
  color?: string;
  width?: number;
  height?: number;
};

const HeartOutline: React.FC<HeartOutlineProps> = ({
  size = 20,
  color = "#000",
  ...props
}) => (
  <svg
    width={size}
    xmlns="http://www.w3.org/2000/svg"
    height={size}
    style={{
      WebkitPrintColorAdjust: "exact",
      paddingTop: 1,
    }}
    fill="none"
    {...props}
  >
    <g
      className="ionicon"
      style={{
        fill: color,
      }}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M13.534 4C11.167 4 10 6.364 10 6.364S8.833 4 6.466 4C4.543 4 3.02 5.63 3 7.575c-.04 4.038 3.162 6.91 6.672 9.323a.578.578 0 0 0 .656 0c3.51-2.413 6.712-5.285 6.672-9.323C16.98 5.63 15.457 4 13.534 4Z"
        style={{
          fill: "none",
        }}
      />
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M13.534 4C11.167 4 10 6.364 10 6.364S8.833 4 6.466 4C4.543 4 3.02 5.63 3 7.575c-.04 4.038 3.162 6.91 6.672 9.323a.578.578 0 0 0 .656 0c3.51-2.413 6.712-5.285 6.672-9.323C16.98 5.63 15.457 4 13.534 4Z"
        style={{
          fill: "none",
          strokeWidth: 2,
          stroke: color,
          strokeOpacity: 1,
        }}
        className="stroke-shape"
      />
    </g>
  </svg>
);

export default HeartOutline;
