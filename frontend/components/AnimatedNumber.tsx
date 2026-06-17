"use client";

import { useEffect, useRef, useState } from "react";
import gsap from "gsap";

type Fmt = "pct" | "num" | "x" | "mps";

/** Tween a metric from its previous value to the new one (GSAP). Tasteful, informative motion. */
export default function AnimatedNumber({
  value,
  format = "num",
  decimals = 2,
}: {
  value: number | null | undefined;
  format?: Fmt;
  decimals?: number;
}) {
  const obj = useRef({ v: 0 });
  const [shown, setShown] = useState(0);

  useEffect(() => {
    if (value == null || !isFinite(value)) return;
    const tween = gsap.to(obj.current, {
      v: value,
      duration: 0.85,
      ease: "power2.out",
      onUpdate: () => setShown(obj.current.v),
    });
    return () => {
      tween.kill();
    };
  }, [value]);

  if (value == null || !isFinite(value)) return <span>—</span>;

  const fmt = (x: number) => {
    switch (format) {
      case "pct":
        return (x * 100).toFixed(decimals) + "%";
      case "x":
        return x.toFixed(decimals) + "×";
      case "mps":
        return x.toFixed(decimals) + "M/s";
      default:
        return x.toFixed(decimals);
    }
  };
  return <span className="tabular-nums">{fmt(shown)}</span>;
}
