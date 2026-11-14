// The one way the app talks to the API: JSON over fetch, the session cookie, and Django's CSRF
// token on every write. Errors come back as ApiError with the status and the server's body.
export class ApiError extends Error {
  constructor(public status: number, public body: any) {
    super(typeof body?.detail === "string" ? body.detail : `Request failed (${status})`);
  }
  /** The first message for a field of a 400 body, e.g. fieldError("name"). */
  fieldError(field: string): string | undefined {
    const v = this.body?.[field];
    return Array.isArray(v) ? v[0] : typeof v === "string" ? v : undefined;
  }
}

let csrf: string | null = null;
async function csrfToken() {
  if (!csrf) csrf = (await (await fetch("/api/auth/csrf", { credentials: "same-origin" })).json()).csrfToken as string;
  return csrf;
}
/** After sign in or out Django rotates the token; the next write fetches it again. */
export const forgetCsrf = () => { csrf = null; };

/** Called with every 401, so the app can return to the sign-in page. */
let onUnauthorized: () => void = () => {};
export const setUnauthorizedHandler = (fn: () => void) => { onUnauthorized = fn; };

export async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (method !== "GET") headers["X-CSRFToken"] = await csrfToken();
  const init: RequestInit = { method, headers, credentials: "same-origin" };
  if (body instanceof FormData) init.body = body;
  else if (body !== undefined) { headers["Content-Type"] = "application/json"; init.body = JSON.stringify(body); }
  const res = await fetch(`/api/${path}`, init);
  const data = res.status === 204 ? null : await res.json().catch(() => null);
  if (!res.ok) {
    if (res.status === 401 && !path.startsWith("auth/")) onUnauthorized();
    if (res.status >= 500) reportError(`${method} /api/${path} answered ${res.status}`);
    throw new ApiError(res.status, data);
  }
  return data as T;
}

export const api = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body?: unknown) => request<T>("POST", path, body ?? {}),
  patch: <T>(path: string, body: unknown) => request<T>("PATCH", path, body),
  put: <T>(path: string, body: unknown) => request<T>("PUT", path, body),
  del: <T>(path: string) => request<T>("DELETE", path),
};

/** Sends a browser-side failure to the server, which logs it and alerts the team. Never throws. */
export function reportError(message: string, detail?: string) {
  try {
    const body = JSON.stringify({ message: message.slice(0, 500), detail: detail?.slice(0, 4000), url: location.pathname });
    navigator.sendBeacon?.("/api/client-errors", new Blob([body], { type: "application/json" })) ||
      fetch("/api/client-errors", { method: "POST", body, headers: { "Content-Type": "application/json" }, keepalive: true }).catch(() => {});
  } catch {
    /* reporting must never break the app */
  }
}
