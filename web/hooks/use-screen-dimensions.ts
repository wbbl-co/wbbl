import { useState, useEffect } from "react";

export function useScreenDimensions(): { width: number; height: number } {
  const [dimensions, setDimensions] = useState<{
    width: number;
    height: number;
  }>({ width: window.innerWidth, height: window.innerHeight });
  useEffect(() => {
    setDimensions({ width: window.innerWidth, height: window.innerHeight });
    const listener = () => {
      setDimensions({ width: window.innerWidth, height: window.innerHeight });
    };
    window.addEventListener("resize", listener);
    return () => {
      window.removeEventListener("resize", listener);
    };
  }, []);

  return dimensions;
}
