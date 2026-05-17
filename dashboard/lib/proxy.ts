export const DEFAULT_PROXY_URL = "http://127.0.0.1:8787";

export function proxyUrl(path: string): URL {
  return new URL(path, process.env.PENNYKITE_PROXY_URL ?? DEFAULT_PROXY_URL);
}
