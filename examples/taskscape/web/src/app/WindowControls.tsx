// The web's WindowControls (replaces tauri/main/src/components/WindowControls.tsx).
// A page has no window to close, minimize or zoom: the browser's own tab and
// window chrome do that, so there are no traffic lights and no Windows caption
// buttons. A panel inside the page (the Trash in the drawer) still closes, with
// an ordinary close button drawn from the tokens.
import { IconButton } from '@taskscape/common-ui/components';

export type WindowControl = 'close' | 'minimize' | 'fullscreen';

export interface WindowControlsProps {
  disabled?: readonly WindowControl[];
}

/** Nothing: the browser draws the window. */
export function WindowControls(_props: WindowControlsProps = {}) {
  return null;
}

export interface DialogControlsProps {
  onClose: () => void;
  closeTitle?: string;
}

export function DialogControls({ onClose, closeTitle = 'Close' }: DialogControlsProps) {
  return (
    <div className="pr-space-4 flex h-full items-center">
      <IconButton
        icon="close"
        size="lg"
        radius="control"
        iconSize={18}
        title={closeTitle}
        aria-label={closeTitle}
        onClick={onClose}
      />
    </div>
  );
}
