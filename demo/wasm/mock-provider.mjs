// Local OpenAI-compatible mock for demo development: the demo's default
// provider (api.illo.fyi) only serves *.illo.fyi origins, so a locally
// hosted page needs this or another CORS-friendly endpoint.
//
//   node mock-provider.mjs [port]   (default 8787)
//
// Then set the demo's base URL to http://localhost:8787/v1.
// Responds to GET /v1/models and POST /v1/chat/completions (including
// stream:true SSE): echoes each entry's source wrapped in guillemets
// for translate requests, or a fixed 0.9 score for judge requests.

import { createServer } from "node:http";

const port = Number(process.argv[2]) || 8787;

const server = createServer((req, res) => {
  res.setHeader("access-control-allow-origin", "*");
  res.setHeader("access-control-allow-headers", "content-type, authorization");
  if (req.method === "OPTIONS") {
    res.writeHead(204).end();
    return;
  }
  if (req.method === "GET" && req.url.endsWith("/models")) {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(
      JSON.stringify({
        data: [
          { id: "illo-mock-translate", object: "model" },
          { id: "illo-mock-judge", object: "model" },
        ],
      }),
    );
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
    // Judge calls name their schema "judgements"; also fall back to the
    // prompt shape when an endpoint strips response_format.
    const isJudge =
      body.response_format?.json_schema?.name === "judgements" ||
      (!/\d+\. "(?:[^"\\]|\\.)*" \(key: [^)]+\)/.test(user) && /key=\("/.test(user));
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
    if (body.stream) {
      // SSE: stream the assembled content in small deltas so the demo's
      // progressive fill has something to render.
      res.writeHead(200, {
        "content-type": "text/event-stream",
        "cache-control": "no-cache",
      });
      const chunks = content.match(/.{1,40}/gs) ?? [];
      let i = 0;
      const tick = () => {
        if (i < chunks.length) {
          res.write(
            `data: ${JSON.stringify({
              choices: [{ delta: { content: chunks[i] } }],
            })}\n\n`,
          );
          i += 1;
          setTimeout(tick, 30);
        } else {
          res.write(
            `data: ${JSON.stringify({ choices: [{ delta: {}, finish_reason: "stop" }] })}\n\n`,
          );
          res.write("data: [DONE]\n\n");
          res.end();
        }
      };
      tick();
      return;
    }
    // Small latency on translate calls so the demo's per-batch progress
    // events are visibly paced.
    const send = () => {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          choices: [{ message: { role: "assistant", content } }],
          model: body.model,
        }),
      );
    };
    if (isJudge) send();
    else setTimeout(send, 300);
  });
});

server.listen(port, () => {
  process.stdout.write(`mock provider on http://localhost:${port}/v1\n`);
});
