export function SpiderLilyMark({ size = 28 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <g stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M24 27c-7-4-12-10-10-17 5 0 10 4 12 9" fill="currentColor" fillOpacity="0.16" />
        <path d="M24 27c7-4 12-10 10-17-5 0-10 4-12 9" fill="currentColor" fillOpacity="0.16" />
        <path d="M24 27c-9 0-15 4-17 10 6 2 12 0 16-4" fill="currentColor" fillOpacity="0.1" />
        <path d="M24 27c9 0 15 4 17 10-6 2-12 0-16-4" fill="currentColor" fillOpacity="0.1" />
        <path d="M24 27V43" />
        <path d="M24 33c-5-3-8-7-9-12" />
        <path d="M24 33c5-3 8-7 9-12" />
        <path d="M24 36c-4 1-7 4-8 8" />
        <path d="M24 36c4 1 7 4 8 8" />
      </g>
      <circle cx="24" cy="27" r="2.2" fill="currentColor" />
    </svg>
  );
}
