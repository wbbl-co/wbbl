import { SVGProps } from "react";

export default function CoreLineZoomOut(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 14 14"
      id="Zoom-Out--Streamline-Core"
      width={"1em"}
      height={"1em"}
      {...props}
    >
      <desc>{"Zoom Out Streamline Icon: https://streamlinehq.com"}</desc>
      <g id="zoom-out--glass-magnifying-out-reduce-zoom">
        <path
          id="Vector"
          stroke="currentcolor"
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M4 6.5h5"
          strokeWidth={1}
        />
        <path
          id="Vector_2"
          stroke="currentcolor"
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M6.5 12.5c3.31371 0 6 -2.68629 6 -6s-2.68629 -6 -6 -6 -6 2.68629 -6 6 2.68629 6 6 6Z"
          strokeWidth={1}
        />
        <path
          id="Vector_3"
          stroke="currentcolor"
          strokeLinecap="round"
          strokeLinejoin="round"
          d="m10.7402 10.7402 2.76 2.76"
          strokeWidth={1}
        />
      </g>
    </svg>
  );
}
