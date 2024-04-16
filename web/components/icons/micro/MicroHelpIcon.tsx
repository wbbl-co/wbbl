import { SVGProps } from "react";
export default function MicroHelpIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 10 10"
      id="Help-Question-Circle--Streamline-Micro"
      width={"1em"}
      height={"1em"}
      {...props}
    >
      <desc>
        {"Help Question Circle Streamline Icon: https://streamlinehq.com"}
      </desc>
      <path
        fill="currentcolor"
        fillRule="evenodd"
        d="M0 5a5 5 0 1 1 10 0A5 5 0 0 1 0 5Zm5 3.047a0.75 0.75 0 1 1 0 -1.5 0.75 0.75 0 0 1 0 1.5Zm-0.75 -4.344a0.75 0.75 0 1 1 0.75 0.75 0.5 0.5 0 0 0 -0.5 0.5v0.672a0.5 0.5 0 0 0 1 0v-0.244a1.75 1.75 0 1 0 -2.25 -1.678 0.5 0.5 0 1 0 1 0Z"
        clipRule="evenodd"
        strokeWidth={1}
      />
    </svg>
  );
}
