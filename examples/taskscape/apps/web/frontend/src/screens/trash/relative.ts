// Short relative time for the row meta line: "30m ago", "3h ago", "5d ago".
export function relativeTime(iso: string, now: number = Date.now()): string {
  const minutes = Math.floor(Math.max(0, now - Date.parse(iso)) / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}
