import { useStore, type Theme, type TodoStore } from "../model.ts";
import { Parts } from "../omnicui/settings.ts";
import { BackIcon } from "./icons.tsx";
import { Switch } from "./Switch.tsx";

const THEMES: { value: Theme; label: string; part: string }[] = [
  { value: "system", label: "System", part: Parts.themeSystem },
  { value: "light", label: "Light", part: Parts.themeLight },
  { value: "dark", label: "Dark", part: Parts.themeDark },
];

/** Settings, a page of its own: the theme (System, Light, Dark) and Show completed. */
export function SettingsPage({ store, onLeave }: { store: TodoStore; onLeave?: () => void }) {
  const state = useStore(store);
  return (
    <main className="settings">
      <button type="button" className="text-button settings__back" data-part={Parts.settingsBack}
        onClick={() => { store.report("closed"); onLeave?.(); }}>
        <BackIcon />Back to tasks
      </button>
      <h1 className="settings__title">Settings</h1>
      <section className="settings__group" aria-labelledby="theme-label">
        <h2 id="theme-label" className="settings__label">Theme</h2>
        <div className="segmented segmented--inline" role="group" aria-labelledby="theme-label">
          {THEMES.map((t) => (
            <button key={t.value} type="button" className="segment" aria-pressed={t.value === state.theme} data-part={t.part} onClick={() => store.chooseTheme(t.value)}>
              {t.label}
            </button>
          ))}
        </div>
      </section>
      <section className="settings__group">
        <Switch label="Show completed" checked={state.showCompleted} onChange={(on) => store.showCompleted(on)} part={Parts.showCompleted} />
        <p className="settings__hint">Off: All lists only the tasks left. Done still lists the done ones.</p>
      </section>
    </main>
  );
}
