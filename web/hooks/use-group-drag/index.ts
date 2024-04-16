import { useRef } from "react";

export default function useGroupDrag() {
  const ref = useRef<SVGElement>();

  return ref;
}
