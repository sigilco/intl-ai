/**
 * SHA-1 hash function using Web Crypto API.
 * Runtime-agnostic: works in Node.js 22+, Bun, browsers, Deno.
 *
 * NOTE: We use SHA-1 only for non-cryptographic content fingerprinting
 * (detecting whether a source string changed). It is NOT used for security.
 */
export async function hashSha1(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buffer = await crypto.subtle.digest("SHA-1", data);
  return Array.from(new Uint8Array(buffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
