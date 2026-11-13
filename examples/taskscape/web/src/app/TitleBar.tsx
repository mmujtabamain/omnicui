// The web's top bar (replaces tauri/main/src/components/TitleBar.tsx). The
// same bar, as the page's header: the project switcher, the list tabs and the
// view buttons, with nothing of a window's (no traffic lights or caption
// buttons, no drag strip or double-click zoom: the browser has the window).
// It adds the quick-capture button, since a page has no menu-bar capture bar.
import { IconButton } from '@taskscape/common-ui/components';
import { formatAccel } from '@taskscape/common-ui/hotkeys';
import { Icon } from '@taskscape/common-ui/Icon';
import { splitTargetId } from '@shared/commands/view';
import { ListTabs } from '@shared/components/ListTabs';
import { ProjectSwitcher } from '@shared/components/ProjectSwitcher';
import { useReducedMotion } from '@shared/lib/reducedMotion';
import { useHotkeyStore } from '@shared/stores/hotkeyStore';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { api } from './api';
import { useWebUi } from './webUi';

export function TitleBar() {
  const reducedMotion = useReducedMotion();
  const splitListId = useLayoutStore((s) => s.splitListId);
  const previewOpen = useLayoutStore((s) => s.previewOpen);
  const hotkeys = useHotkeyStore((s) => s.map);
  useLayoutStore((s) => s.activeListId);
  useListStore((s) => s.lists);
  const splitTarget = splitTargetId();

  const hint = (id: string) => {
    const accel = formatAccel(hotkeys[id] ?? '');
    return accel ? `  ${accel}` : '';
  };

  const toggleSplit = () => {
    if (splitTarget) useLayoutStore.getState().toggleSplit(splitTarget);
  };

  return (
    <header
      data-part="titlebar"
      className="border-edge-2l dark:border-edge-2d bg-surface-1l dark:bg-surface-1d relative flex h-13 shrink-0 items-stretch border-b"
    >
      <div className="flex items-center pr-4 pl-3.5">
        <ProjectSwitcher />
      </div>

      <nav aria-label="Lists" className="flex min-w-0 flex-1 items-stretch">
        <ListTabs />
      </nav>

      <div className="gap-space-4 pr-space-6 pl-space-6 flex items-center">
        {reducedMotion && (
          <div
            title="Reduced motion is on (Settings > General)"
            className="text-content-3l dark:text-content-3d grid h-8 w-8 shrink-0 place-items-center"
          >
            <Icon name="motion_photos_off" size={19} weight={300} />
          </div>
        )}
        <BarButton
          part="quickCapture"
          icon="bolt"
          title={`Quick capture${hint('toggle_capture_bar')}`}
          onClick={() => useWebUi.getState().setCaptureOpen(true)}
        />
        {splitListId != null && (
          <BarButton
            part="swapPanes"
            icon="swap_horiz"
            title="Swap panes"
            onClick={() => useLayoutStore.getState().swapPanes()}
          />
        )}
        <BarButton
          part="splitView"
          icon="vertical_split"
          active={splitListId != null}
          disabled={splitTarget == null}
          title={splitListId ? 'Close split view' : 'Split view'}
          onClick={toggleSplit}
        />
        <BarButton
          part="togglePreview"
          icon={previewOpen ? 'right_panel_close' : 'right_panel_open'}
          active={previewOpen}
          aria-expanded={previewOpen}
          title={`${previewOpen ? 'Hide' : 'Show'} preview panel${hint('toggle_preview')}`}
          onClick={() => useLayoutStore.getState().togglePreview()}
        />
        <BarButton
          part="openSettings"
          icon="settings"
          aria-haspopup="dialog"
          title={`Settings${hint('open_settings')}`}
          onClick={() => void api.openSettings()}
        />
      </div>
    </header>
  );
}

function BarButton({
  part,
  icon,
  title,
  onClick,
  active = false,
  disabled = false,
  ...aria
}: {
  part: string;
  icon: string;
  title: string;
  onClick: () => void;
  active?: boolean;
  disabled?: boolean;
  'aria-expanded'?: boolean;
  'aria-haspopup'?: 'dialog';
}) {
  return (
    <IconButton
      data-part={part}
      icon={icon}
      size="xl"
      radius="control"
      iconSize={19}
      iconWeight={300}
      filled={active}
      active={active}
      disabled={disabled}
      onClick={onClick}
      title={title}
      {...aria}
    />
  );
}
