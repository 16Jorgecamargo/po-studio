const common = {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.8,
  strokeLinecap: 'round' as const,
  strokeLinejoin: 'round' as const,
  'aria-hidden': true,
};

export function SunIcon() {
  return (
    <svg {...common}>
      <circle cx="12" cy="12" r="4.2" />
      <path d="M12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M5.5 18.5l1.7-1.7M16.8 7.2l1.7-1.7" />
    </svg>
  );
}

export function MoonIcon() {
  return (
    <svg {...common}>
      <path d="M19.5 14.6A7.8 7.8 0 0 1 9.4 4.5a7.8 7.8 0 1 0 10.1 10.1Z" />
    </svg>
  );
}

export function MotionIcon({ off }: { off: boolean }) {
  return (
    <svg {...common}>
      <path d="M3.5 15c2.5-6 5-6 7.5 0s5 6 9.5-1" />
      {off ? <path d="M4 4l16 16" /> : null}
    </svg>
  );
}

export function ChevronIcon() {
  return (
    <svg {...common} width="16" height="16">
      <path d="M6 9.5l6 6 6-6" />
    </svg>
  );
}

export function CopyIcon() {
  return (
    <svg {...common} width="16" height="16">
      <rect x="8.5" y="8.5" width="11" height="11" rx="2" />
      <path d="M15.5 8.5V6a1.5 1.5 0 0 0-1.5-1.5H6A1.5 1.5 0 0 0 4.5 6v8A1.5 1.5 0 0 0 6 15.5h2.5" />
    </svg>
  );
}
