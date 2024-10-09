import * as React from "react";

export type AddProps = {
  size?: number;
  color?: string;
};

const Add: React.FC<AddProps> = ({ size = 17, color = "#000", ...props }) => (
  <svg
    width={size}
    xmlns="http://www.w3.org/2000/svg"
    height={size}
    style={{
      WebkitPrintColorAdjust: "exact",
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
        d="M11 5v12m6-6H5"
        style={{
          fill: "none",
        }}
      />
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M11 5v12m6-6H5"
        style={{
          fill: "none",
          strokeWidth: 2,
          stroke: color,
        }}
        className="stroke-shape"
      />
    </g>
  </svg>
);

export default Add;
