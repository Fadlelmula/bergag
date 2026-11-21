import * as Switch from "@radix-ui/react-switch";
import { useId } from "react";

interface SwitchFieldProps {
  label: string;
  hint?: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function SwitchField({ label, hint, checked, onCheckedChange, disabled }: SwitchFieldProps) {
  const id = useId();
  return (
    <div className="bergag-switch-row">
      <div className="bergag-switch-row__text">
        <label className="bergag-switch-row__label" htmlFor={id}>
          {label}
        </label>
        {hint ? <span className="bergag-switch-row__hint">{hint}</span> : null}
      </div>
      <Switch.Root
        id={id}
        className="bergag-switch"
        checked={checked}
        onCheckedChange={onCheckedChange}
        disabled={disabled}
        aria-label={label}
      >
        <Switch.Thumb className="bergag-switch__thumb" />
      </Switch.Root>
    </div>
  );
}
