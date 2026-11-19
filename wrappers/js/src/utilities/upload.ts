import { QuantionError } from "./errors";
import type { HttpClient } from "./httpClient";

export async function uploadIon(
  url: string,
  client: HttpClient,
  bytes: Uint8Array,
): Promise<void> {
  const response = await client.put(url, bytes).catch((error: Error) => {
    throw new QuantionError(
      "upload_failed",
      "writeIon",
      `PUT failed: ${error.message}`,
    );
  });
  if (!response.ok) {
    throw new QuantionError(
      "upload_failed",
      "writeIon",
      `PUT returned ${response.status}`,
    );
  }
}
