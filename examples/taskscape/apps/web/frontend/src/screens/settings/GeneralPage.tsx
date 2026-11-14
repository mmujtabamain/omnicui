// The General pane: Appearance (theme, reduced motion) and Tasks (show completed tasks).
import { Switch } from "@taskscape/ui/switch";
import type { Settings } from "../../api/types";
import { Choice, PrefField, Segments, SettingItem, SettingsGroup, SearchRow, rowMatches } from "./shared";

/** The rows this pane contributes to search. */
export const GENERAL_ROWS: SearchRow[] = [
  {
    id: "theme",
    pane: "general",
    group: "appearance",
    title: "Theme",
    description: "Follow the system appearance, or pin one.",
    keywords: "dark light color colour mode",
  },
  {
    id: "reducedMotion",
    pane: "general",
    group: "appearance",
    title: "Reduced motion",
    description: "Cuts animations and transitions to the minimum.",
    keywords: "animation accessibility reduce transition",
  },
  {
    id: "showCompleted",
    pane: "general",
    group: "tasks",
    title: "Show completed tasks",
    description: "Keep finished tasks in the list instead of hiding them.",
    keywords: "done finished hidden",
  },
];

const APPEARANCE: PrefField[] = ["theme", "reduced_motion"];
const TASKS: PrefField[] = ["show_completed"];
const SAVE_ERROR = "Could not save. Try again.";

const THEME_CHOICES: Choice<Settings["theme"]>[] = [
  { value: "system", label: "System", icon: "contrast", part: "app.settingsgeneral.themeSystem" },
  { value: "light", label: "Light", icon: "light_mode", part: "app.settingsgeneral.themeLight" },
  { value: "dark", label: "Dark", icon: "dark_mode", part: "app.settingsgeneral.themeDark" },
];

const MOTION_CHOICES: Choice<Settings["reduced_motion"]>[] = [
  { value: "system", label: "System", part: "app.settingsgeneral.reducedMotionSystem" },
  { value: "on", label: "On", part: "app.settingsgeneral.reducedMotionOn" },
  { value: "off", label: "Off", part: "app.settingsgeneral.reducedMotionOff" },
];

export interface GeneralPageProps {
  /** The values now shown. */
  values: Settings;
  /** The values when the page opened. */
  baseline: Settings;
  /** Words of the search; empty when not searching. */
  tokens: string[];
  /** The search is limited to this pane (a click on a tab that had matches). */
  scoped: boolean;
  failed: Partial<Record<PrefField, boolean>>;
  onChange: (patch: Partial<Settings>) => void;
  onDiscard: (fields: PrefField[]) => void;
}

export function GeneralPage({ values, baseline, tokens, scoped, failed, onChange, onDiscard }: GeneralPageProps) {
  const searching = tokens.length > 0;
  const rowShown = (id: string) => !searching || GENERAL_ROWS.some((row) => row.id === id && rowMatches(row, tokens));
  const groupShown = (group: string) =>
    !searching || GENERAL_ROWS.some((row) => row.group === group && rowMatches(row, tokens));
  const heading = (name: string) => (searching && !scoped ? `General · ${name}` : name);
  const changed = (fields: PrefField[]) => fields.some((field) => values[field] !== baseline[field]);

  return (
    <div className="flex min-w-0 flex-col gap-[1.286rem]">
      <SettingsGroup
        label={heading("Appearance")}
        labelPart="app.settingsgeneral.appearanceHeader"
        groupPart="app.settingsgeneral.appearanceGroup"
        discardPart="app.settingsgeneral.appearanceDiscard"
        changed={changed(APPEARANCE)}
        onDiscard={() => onDiscard(APPEARANCE)}
        visible={groupShown("appearance")}
      >
        <SettingItem
          title="Theme"
          titlePart="app.settingsgeneral.themeTitle"
          description="Follow the system appearance, or pin one."
          descriptionPart="app.settingsgeneral.themeDescription"
          visible={rowShown("theme")}
          stacked
          error={failed.theme ? SAVE_ERROR : ""}
        >
          <Segments
            label="Theme"
            stacked
            value={values.theme}
            choices={THEME_CHOICES}
            onChange={(theme) => onChange({ theme })}
          />
        </SettingItem>
        <SettingItem
          title="Reduced motion"
          titlePart="app.settingsgeneral.reducedMotionTitle"
          description="Cuts animations and transitions to the minimum."
          descriptionPart="app.settingsgeneral.reducedMotionDescription"
          visible={rowShown("reducedMotion")}
          error={failed.reduced_motion ? SAVE_ERROR : ""}
        >
          <Segments
            label="Reduced motion"
            value={values.reduced_motion}
            choices={MOTION_CHOICES}
            onChange={(reduced_motion) => onChange({ reduced_motion })}
          />
        </SettingItem>
      </SettingsGroup>
      <SettingsGroup
        label={heading("Tasks")}
        labelPart="app.settingsgeneral.tasksHeader"
        groupPart="app.settingsgeneral.tasksGroup"
        discardPart="app.settingsgeneral.tasksDiscard"
        changed={changed(TASKS)}
        onDiscard={() => onDiscard(TASKS)}
        visible={groupShown("tasks")}
      >
        <SettingItem
          title="Show completed tasks"
          titlePart="app.settingsgeneral.showCompletedTitle"
          description="Keep finished tasks in the list instead of hiding them."
          descriptionPart="app.settingsgeneral.showCompletedDescription"
          visible={rowShown("showCompleted")}
          error={failed.show_completed ? SAVE_ERROR : ""}
        >
          <Switch
            data-part="app.settingsgeneral.showCompletedSwitch"
            ariaLabel="Show completed tasks"
            checked={values.show_completed}
            onCheckedChange={(show_completed) => onChange({ show_completed })}
          />
        </SettingItem>
      </SettingsGroup>
    </div>
  );
}
