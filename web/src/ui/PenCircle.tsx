import { useMemo } from 'react';
import { penLoop } from './pen';

interface Props {
  seed: string;
  className?: string;
}

export function PenCircle({ seed, className }: Props) {
  const d = useMemo(() => penLoop(seed, 50, 50, 46, 44), [seed]);
  return (
    <svg className={`pen-circle ${className ?? ''}`} viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
      <path d={d} pathLength={1} vectorEffect="non-scaling-stroke" />
    </svg>
  );
}
