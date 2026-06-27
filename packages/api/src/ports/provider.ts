export interface AIProvider {
  readonly id: string;
  buildRequest(opts: {
    model: string;
    systemPrompt: string;
    userPrompt: string;
    temperature: number;
    modelParams?: Record<string, unknown>;
  }): {
    url: string;
    headers: Record<string, string>;
    body: Record<string, unknown>;
  };
  parseResponse(data: unknown): { content: string };
}
