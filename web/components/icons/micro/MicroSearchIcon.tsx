import { SVGProps } from "react";

export default function MicroSearchIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 10 10"
      id="Search--Streamline-Micro"
      width={"1em"}
      height={"1em"}
      {...props}
    >
      <desc>{"Search Streamline Icon: https://streamlinehq.com"}</desc>
      <path
        fill="currentcolor"
        fillRule="evenodd"
        d="M1.5 4.528a3.028 3.028 0 1 1 6.056 0 3.028 3.028 0 0 1 -6.056 0Zm5.655 3.688a4.528 4.528 0 1 1 1.06 -1.06L9.78 8.72a0.75 0.75 0 1 1 -1.06 1.06L7.155 8.216Z"
        clipRule="evenodd"
        strokeWidth={1}
      />
    </svg>
  );
}
