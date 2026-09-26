import type { AIProvider } from "../../ports/provider";

export const anthropicProvider: AIProvider = {
  id: "anthropic",
  buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
    return {
      url: "/messages",
      headers: {
        "Content-Type": "application/json",
        "anthropic-version": "2023-06-01",
        "X-Api-Key": "${apiKey}",
      },
      body: {
        model,
        max_tokens: (modelParams?.max_tokens as number) ?? 1024,
        system: [{ type: "text" as const, text: systemPrompt }],
        messages: [{ role: "user", content: userPrompt }],
        temperature,
        ...modelParams,
      },
    };
  },
  parseResponse(data: unknown) {
    return {
      content: (data as { content?: Array<{ text?: string }> }).content?.[0]?.text ?? "",
    };
  },
};
