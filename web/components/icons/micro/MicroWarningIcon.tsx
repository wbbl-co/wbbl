import { SVGProps } from "react";

export default function MicroWarniningIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 10 10"
      id="Warning-Triangle--Streamline-Micro"
      width={"1em"}
      height={"1em"}
      {...props}
    >
      <desc>
        {"Warning Triangle Streamline Icon: https://streamlinehq.com"}
      </desc>
      <path
        fill="currentcolor"
        fillRule="evenodd"
        d="M4.275 0.198a1.422 1.422 0 0 1 1.972 0.54l3.5 6.28 0.002 0.003A2.01 2.01 0 0 1 7.988 10H2.01A2.008 2.008 0 0 1 0.251 7.024l0.001 -0.002L3.753 0.737a1.43 1.43 0 0 1 0.522 -0.539ZM5.5 3.25a0.5 0.5 0 0 0 -1 0v2a0.5 0.5 0 0 0 1 0v-2ZM5 6.576a0.75 0.75 0 1 1 0 1.5 0.75 0.75 0 0 1 0 -1.5Z"
        clipRule="evenodd"
        strokeWidth={1}
      />
    </svg>
  );
}
