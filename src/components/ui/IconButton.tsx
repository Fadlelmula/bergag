import { Button, type ButtonProps } from "./Button";
import type { ReactNode } from "react";

export interface IconButtonProps extends Omit<ButtonProps, "children" | "icon"> {
  label: string;
  icon: ReactNode;
}

export function IconButton({ label, icon, ...props }: IconButtonProps) {
  return (
    <Button
      aria-label={label}
      title={label}
      variant="ghost"
      size="sm"
      className="bergag-icon-button"
      icon={icon}
      {...props}
    />
  );
}
