import { useEffect, useRef, useState } from 'react';

// Observe a container element and report its size so scopes can fill
// their panel instead of rendering at a fixed resolution.
export function useCanvasSize(minSize = 160) {
  const ref = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 400, height: 300 });

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const ro = new ResizeObserver((entries) => {
      const rect = entries[0].contentRect;
      setSize({
        width: Math.max(minSize, Math.floor(rect.width)),
        height: Math.max(minSize, Math.floor(rect.height)),
      });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [minSize]);

  return { ref, size };
}
