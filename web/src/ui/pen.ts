function hash(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function rng(seed: number) {
  let s = seed || 1;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return ((s >>> 0) % 10000) / 10000;
  };
}

export function penLoop(seed: string, cx: number, cy: number, rx: number, ry: number): string {
  const rand = rng(hash(seed));
  const start = -Math.PI * (0.55 + rand() * 0.3);
  const sweep = Math.PI * 2 + 0.35 + rand() * 0.3;
  const p1 = rand() * Math.PI * 2;
  const p2 = rand() * Math.PI * 2;
  const tilt = (rand() - 0.5) * 0.12;
  const steps = 56;
  const pts: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const a = start + sweep * t;
    const wobble = 1 + 0.035 * Math.sin(a * 2 + p1) + 0.025 * Math.sin(a * 3 + p2) + 0.05 * t;
    const x = Math.cos(a) * rx * wobble;
    const y = Math.sin(a) * ry * wobble;
    const xr = x * Math.cos(tilt) - y * Math.sin(tilt) + cx;
    const yr = x * Math.sin(tilt) + y * Math.cos(tilt) + cy;
    pts.push(`${xr.toFixed(2)} ${yr.toFixed(2)}`);
  }
  return `M ${pts[0]} L ${pts.slice(1).join(' L ')}`;
}
