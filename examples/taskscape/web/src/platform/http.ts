// The one place the page talks to server/. Counts what is in flight, so a
// caller can tell whether the app is still waiting on the server
// (`requestsInFlight`, which the omnicui harness reads as the app's busy).

let inFlight = 0;

export const requestsInFlight = () => inFlight;

/** A JSON request to `/api<path>`. A File or Blob body is sent as it is (an
 *  upload). Resolves with the parsed answer (undefined for 204); rejects with
 *  the server's message, a plain string. */
export async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  inFlight++;
  try {
    const raw = body instanceof Blob;
    const res = await fetch(`/api${path}`, {
      method,
      headers: body === undefined ? undefined : raw ? { 'content-type': 'application/octet-stream' } : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : raw ? body : JSON.stringify(body),
    });
    if (!res.ok) {
      const text = await res.text().catch(() => '');
      if (res.status >= 500) console.error(`[taskscape] ${method} /api${path} failed (${res.status}): ${text}`);
      throw text || `${method} ${path}: ${res.status} ${res.statusText}`;
    }
    if (res.status === 204) return undefined as T;
    const type = res.headers.get('content-type') ?? '';
    return (type.includes('json') ? await res.json() : await res.text()) as T;
  } catch (e) {
    // fetch itself failed: the server is down or unreachable.
    if (e instanceof TypeError) throw `The server can't be reached (${e.message})`;
    throw e;
  } finally {
    inFlight--;
  }
}
