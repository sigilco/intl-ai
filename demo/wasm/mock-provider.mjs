// Local OpenAI-compatible mock for demo development: the demo's default
// provider (api.illo.fyi) only serves *.illo.fyi origins, so a locally
// hosted page needs this or another CORS-friendly endpoint.
//
//   node mock-provider.mjs [port]   (default 8787)
//
// Then set the demo's base URL to http://localhost:8787/v1.
// Responds to POST /v1/chat/completions: echoes each entry's source
// wrapped in guillemets for translate requests, or a fixed 0.9 score
// for judge requests.

import { createServer } from "node:http";

const port = Number(process.argv[2]) || 8787;

const server = createServer((req, res) => {
  res.setHeader("access-control-allow-origin", "*");
  res.setHeader("access-control-allow-headers", "content-type");
  if (req.method === "OPTIONS") {
    res.writeHead(204).end();
    return;
  }
  if (req.method !== "POST" || !req.url.endsWith("/chat/completions")) {
    res.writeHead(404).end('{"error":"not found"}');
    return;
  }
  let raw = "";
  req.on("data", (chunk) => {
    raw += chunk;
  });
  req.on("end", () => {
    const body = JSON.parse(raw);
    const user = body.messages?.find((m) => m.role === "user")?.content ?? "";
    const isJudge = body.response_format?.json_schema?.name === "judgements";
    let content;
    if (isJudge) {
      const keys = [...user.matchAll(/key=("[^"]*")/g)].map((m) => JSON.parse(m[1]));
      content = JSON.stringify({
        judgements: keys.map((key) => ({
          key,
          score: 0.9,
          reason: "mock provider",
          errors: [],
        })),
      });
    } else {
      const rows = [...user.matchAll(/\d+\. ("(?:[^"\\]|\\.)*") \(key: ([^)]+)\)/g)];
      content = JSON.stringify({
        translations: rows.map(([, src, key]) => ({
          key,
          translated: `«${JSON.parse(src)}»`,
        })),
      });
    }
    res.writeHead(200, { "content-type": "application/json" });
    res.end(
      JSON.stringify({
        choices: [{ message: { role: "assistant", content } }],
        model: body.model,
      }),
    );
  });
});

server.listen(port, () => {
  process.stdout.write(`mock provider on http://localhost:${port}/v1\n`);
});
