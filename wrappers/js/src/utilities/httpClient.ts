import type { Header } from "../types/types";

export class HttpClient {
  private readonly headers: Record<string, string>;

  constructor(headers: Header[] = []) {
    this.headers = Object.fromEntries(
      headers.map((header) => [header.name.trim(), header.value]),
    );
  }

  send(url: string, init: RequestInit = {}): Promise<Response> {
    return fetch(url, {
      ...init,
      headers: { ...this.headers, ...(init.headers as Record<string, string>) },
    });
  }

  getRange(url: string, spec: string): Promise<Response> {
    return this.send(url, {
      headers: { Range: `bytes=${spec}`, "Accept-Encoding": "identity" },
    });
  }

  put(url: string, bytes: Uint8Array): Promise<Response> {
    return this.send(url, {
      method: "PUT",
      body: new Blob([bytes as BlobPart]),
    });
  }
}
