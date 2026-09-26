import type { AIProvider } from "../../ports/provider";

const TRANSACTION_SCHEMA = {
  type: "object",
  properties: {
    translations: {
      type: "array",
      items: {
        type: "object",
        properties: {
          key: { type: "string" },
          translated: { type: "string" },
        },
        required: ["key", "translated"],
        additionalProperties: false,
      },
    },
  },
  required: ["translations"],
  additionalProperties: false,
};

export const openaiProvider: AIProvider = {
  id: "openai",
  buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
    return {
      url: "/chat/completions",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer ${apiKey}",
      },
      body: {
        model,
        messages: [
          { role: "system", content: systemPrompt },
          { role: "user", content: userPrompt },
        ],
        response_format: {
          type: "json_schema",
          json_schema: {
            name: "translations",
            schema: TRANSACTION_SCHEMA,
          },
        },
        temperature,
        ...modelParams,
      },
    };
  },
  parseResponse(data: unknown) {
    let content =
      (data as { choices?: Array<{ message?: { content?: string } }> }).choices?.[0]?.message
        ?.content ?? "";
    // Strip markdown code fences — providers that ignore response_format emit ```json fences
    content = content.replace(/^[`]{3,}.*$/gm, "").trim();
    return { content };
  },
};
