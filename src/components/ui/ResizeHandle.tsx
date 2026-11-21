import { useRef, type PointerEvent as ReactPointerEvent, type KeyboardEvent } from "react";

interface ResizeHandleProps {
  label: string;
  value: number;
  min: number;
  max: number;
  onDelta: (delta: number) => void;
  onCommit: () => void;
  interactive?: boolean;
}

export function ResizeHandle({
  label,
  value,
  min,
  max,
  onDelta,
  onCommit,
  interactive = true,
}: ResizeHandleProps) {
  const dragging = useRef(false);
  const lastX = useRef(0);

  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!interactive) return;
    event.preventDefault();
    dragging.current = true;
    lastX.current = event.clientX;
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragging.current) return;
    const delta = event.clientX - lastX.current;
    lastX.current = event.clientX;
    if (delta !== 0) onDelta(delta);
  };

  const handlePointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragging.current) return;
    dragging.current = false;
    event.currentTarget.releasePointerCapture(event.pointerId);
    onCommit();
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!interactive) return;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    onDelta(event.key === "ArrowRight" ? 12 : -12);
    onCommit();
  };

  return (
    <div
      className="bergag-resize-handle"
      role="separator"
      aria-label={label}
      aria-orientation="vertical"
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={Math.round(value)}
      aria-valuetext={`${Math.round(value)} px`}
      aria-hidden={interactive ? undefined : true}
      tabIndex={interactive ? 0 : -1}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      onKeyDown={handleKeyDown}
    >
      <span aria-hidden="true" />
    </div>
  );
}
