// The web's PathButton (replaces tauri/main/src/windows/settings/PathButton.tsx).
// A page can't open a folder in the file manager, so a data location is shown
// as text the user can select, with a button that copies it.
import { IconButton, Label } from '@taskscape/common-ui/components';
import { useState } from 'react';

export interface PathButtonProps {
  path: string | null;
}

export function PathButton({ path }: PathButtonProps) {
  const [copied, setCopied] = useState(false);
  if (!path)
    return (
      <Label tone="muted" className="text-[11.5px]">
        —
      </Label>
    );
  return (
    <div className="gap-space-3 flex max-w-full items-center self-start">
      <span className="selectable-text text-content-2l dark:text-content-2d truncate font-mono text-[11.5px]">
        {path}
      </span>
      <IconButton
        icon={copied ? 'check' : 'content_copy'}
        iconSize={14}
        title={copied ? 'Copied' : 'Copy the path'}
        aria-label={`Copy ${path}`}
        onClick={() =>
          void navigator.clipboard.writeText(path).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          })
        }
      />
    </div>
  );
}
