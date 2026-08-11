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

/**
 * Drives a local headless coding agent (subprocess) as a translation backend
 * instead of an HTTP API. No model/temperature/modelParams: no headless agent
 * CLI accepts them, so including them here would be a lie in the type.
 */
export interface AITransport {
  readonly id: string;
  complete(opts: {
    systemPrompt: string;
    userPrompt: string;
    signal: AbortSignal;
  }): Promise<{ content: string }>;
}
