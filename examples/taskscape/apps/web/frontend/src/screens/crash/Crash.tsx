import { Button } from "@taskscape/ui/button";

/** Shown when a screen crashes: the error has already been reported. */
export function Crash({ error }: { error: Error }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 bg-surface-1 text-content-2">
      <div className="text-[1.143rem] font-semibold text-content-1">Something went wrong</div>
      <div className="max-w-96 text-center">{error.message}</div>
      <Button data-part="app.crash.reload" variant="primary" size="lg" width="content" label="Reload" onClick={() => location.reload()} />
    </div>
  );
}
