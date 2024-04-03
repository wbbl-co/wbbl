import { useCallback } from "react";

export function andThen<Func extends Function>(f: Func, g: Func): Func {
  return useCallback(
    function combined() {
      if (f) {
        f(...arguments);
      }
      if (g) {
        return g(...arguments);
      }
    } as unknown as Func,
    [f, g],
  );
}
