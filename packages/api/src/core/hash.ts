/**
 * SHA-1 hash for non-cryptographic content fingerprinting (staleness detection).
 * Runtime-agnostic: Node.js 22+, Bun, browsers, Deno.
 */
export async function hashSha1(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buffer = await crypto.subtle.digest("SHA-1", data);
  return Array.from(new Uint8Array(buffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
