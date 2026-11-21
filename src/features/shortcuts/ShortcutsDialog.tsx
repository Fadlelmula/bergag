import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { IconButton } from "../../components/ui/IconButton";

export interface ShortcutRow {
  keys: string;
  description: string;
  status: "available" | "planned";
  milestone?: number;
}

export const SHORTCUTS: ShortcutRow[] = [
  { keys: "J", description: "Next article in the current list", status: "available" },
  { keys: "K", description: "Previous article in the current list", status: "available" },
  { keys: "Enter", description: "Open the focused article row", status: "available" },
  { keys: "M", description: "Toggle read state", status: "available" },
  { keys: "S", description: "Save / remove read later", status: "available" },
  { keys: "V", description: "Open original website", status: "available" },
  { keys: "F", description: "Toggle reader focus mode", status: "available" },
  { keys: "/", description: "Focus settings search", status: "available" },
  { keys: "G then H", description: "Go to Home", status: "available" },
  { keys: "G then S", description: "Open Settings", status: "available" },
  { keys: "?", description: "Show keyboard shortcuts", status: "available" },
  { keys: "Esc", description: "Close dialogs", status: "available" },
];

export function ShortcutsDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content className="bergag-dialog" style={{ width: 560, height: 560 }}>
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  Keyboard shortcuts
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  List navigation, article state, focus mode, and shell shortcuts are active now.
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close shortcuts" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">
              {SHORTCUTS.map((shortcut) => (
                <div className="bergag-key-row" key={shortcut.keys}>
                  <span className="bergag-key">{shortcut.keys}</span>
                  <span className="bergag-key-description">{shortcut.description}</span>
                  {shortcut.status === "available" ? (
                    <span className="bergag-badge bergag-badge--accent">Active</span>
                  ) : (
                    <span className="bergag-badge">M{shortcut.milestone}</span>
                  )}
                </div>
              ))}
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
