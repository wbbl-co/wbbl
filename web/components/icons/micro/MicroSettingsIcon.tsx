import { SVGProps } from "react";

export default function MicroSettingsIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 10 10"
      id="Setting-Toggle--Streamline-Micro"
      width={"1em"}
      height={"1em"}
      {...props}
    >
      <desc>{"Setting Toggle Streamline Icon: https://streamlinehq.com"}</desc>
      <path
        fill="currentcolor"
        fillRule="evenodd"
        d="M2.25 0h5.5a2.25 2.25 0 0 1 0 4.5h-5.5a2.25 2.25 0 1 1 0 -4.5Zm4.306 3.29a1.25 1.25 0 1 0 1.388 -2.08 1.25 1.25 0 0 0 -1.388 2.08ZM2.25 5.5h5.5a2.25 2.25 0 1 1 0 4.5h-5.5a2.25 2.25 0 1 1 0 -4.5Zm-0.194 3.29a1.25 1.25 0 1 0 1.388 -2.08 1.25 1.25 0 0 0 -1.388 2.08Z"
        clipRule="evenodd"
        strokeWidth={1}
      />
    </svg>
  );
}
