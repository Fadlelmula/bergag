import * as Dialog from "@radix-ui/react-dialog";
import { ClipboardCopy, LoaderCircle, Upload, X } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { OpmlImportResult } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface OpmlDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImported: () => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function ImportOpmlDialog({ open, onOpenChange, onImported }: OpmlDialogProps) {
  const [xml, setXml] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<OpmlImportResult | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const loadFile = async (file: File | undefined) => {
    if (!file) return;
    setError(null);
    try {
      setXml(await file.text());
    } catch (caught) {
      setError(errorMessage(caught));
    }
  };

  const importFile = async () => {
    if (!xml.trim()) {
      setError("Choose an OPML file or paste OPML XML first.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const imported = await desktop.importOpml(xml);
      setResult(imported);
      onImported();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content
          className="bergag-dialog"
          style={{ width: "min(620px, calc(100vw - 48px))", height: "auto" }}
        >
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  Import OPML
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  Feeds are fetched and validated before they are added. Existing subscriptions are
                  skipped.
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close import" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">
              <input
                ref={fileRef}
                type="file"
                accept=".opml,.xml,text/xml,application/xml"
                style={{ display: "none" }}
                onChange={(event) => void loadFile(event.target.files?.[0])}
              />
              <div className="bergag-inline-actions" style={{ marginBottom: 12 }}>
                <Button variant="secondary" onClick={() => fileRef.current?.click()}>
                  Choose .opml file
                </Button>
                <span className="bergag-field__hint">or paste XML below</span>
              </div>
              <textarea
                className="bergag-input"
                aria-label="OPML XML"
                rows={10}
                value={xml}
                placeholder='<opml version="2.0">…</opml>'
                onChange={(event) => setXml(event.target.value)}
                style={{ fontFamily: "Consolas, monospace", fontSize: 11, resize: "vertical" }}
              />
              {error ? <p className="bergag-error">{error}</p> : null}
              {result ? (
                <div className="bergag-note" style={{ marginTop: 12 }}>
                  Imported {result.imported}, skipped {result.skipped}.
                  {result.errors.length > 0 ? (
                    <ul className="bergag-bullet-list" style={{ marginTop: 8 }}>
                      {result.errors.slice(0, 10).map((entry) => (
                        <li key={entry}>{entry}</li>
                      ))}
                    </ul>
                  ) : null}
                </div>
              ) : null}
              <div className="bergag-inline-actions" style={{ marginTop: 16 }}>
                <Button
                  variant="primary"
                  onClick={() => void importFile()}
                  disabled={busy || !xml.trim()}
                  icon={busy ? <LoaderCircle size={14} /> : <Upload size={14} />}
                >
                  Import
                </Button>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export function ExportOpmlDialog({ open, onOpenChange }: Omit<OpmlDialogProps, "onImported">) {
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState<string | null>(null);
  const exportQuery = useQuery({
    queryKey: ["feeds", "opml", "export"],
    queryFn: desktop.exportOpml,
    enabled: open,
    staleTime: 0,
  });
  const xml = exportQuery.data ?? "";

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(xml);
      setCopied(true);
    } catch (caught) {
      setCopyError(errorMessage(caught));
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content
          className="bergag-dialog"
          style={{ width: "min(620px, calc(100vw - 48px))", height: "auto" }}
        >
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  Export OPML
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  This file contains your feed URLs and folder names. Keep it private.
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close export" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">
              <textarea
                className="bergag-input"
                aria-label="OPML export"
                rows={12}
                readOnly
                value={xml}
                style={{ fontFamily: "Consolas, monospace", fontSize: 11, resize: "vertical" }}
              />
              {exportQuery.isLoading ? (
                <p className="bergag-field__hint">Generating OPML…</p>
              ) : null}
              {exportQuery.error ? (
                <p className="bergag-error">{errorMessage(exportQuery.error)}</p>
              ) : null}
              {copyError ? <p className="bergag-error">{copyError}</p> : null}
              <div className="bergag-inline-actions" style={{ marginTop: 16 }}>
                <Button
                  variant="primary"
                  onClick={() => void copy()}
                  disabled={!xml}
                  icon={<ClipboardCopy size={14} />}
                >
                  {copied ? "Copied" : "Copy OPML"}
                </Button>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
