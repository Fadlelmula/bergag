import * as Tooltip from "@radix-ui/react-tooltip";
import type { ReactNode } from "react";

export function Hint({
  content,
  children,
  side = "bottom",
}: {
  content: ReactNode;
  children: ReactNode;
  side?: "top" | "right" | "bottom" | "left";
}) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="bergag-tooltip" side={side} sideOffset={6}>
          {content}
          <Tooltip.Arrow className="bergag-tooltip__arrow" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}
