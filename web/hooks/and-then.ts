import { useCallback } from "react";

export function andThen<Func extends (...args: any[]) => void>(
  f: Func,
  g: Func,
): Func {
  return useCallback(
    function combined(...rest: any[]) {
      if (f) {
        f(...rest);
      }
      if (g) {
        return g(...rest);
      }
    } as unknown as Func,
    [f, g],
  );
}
