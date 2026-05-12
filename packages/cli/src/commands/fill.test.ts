import { describe, test, expect } from "vitest";
import { Command } from "commander";

// Simple unit tests for fill command
describe("fill command", () => {
  test("module can be imported without error", async () => {
    // This test verifies the module can be loaded
    const { fillCommand, fillAction } = await import("./fill");

    // Verify exports exist
    expect(fillCommand).toBeDefined();
    expect(fillAction).toBeDefined();
    expect(fillCommand).toBeInstanceOf(Command);
  });

  test("command options are configured correctly", async () => {
    const { fillCommand } = await import("./fill");
    const options = fillCommand.options;

    // Check that the expected options exist
    expect(options).toBeDefined();

    // Verify options array contains expected flags
    const optionNames = options.map((opt: any) => opt.long);
    expect(optionNames).toContain("--dry-run");
    expect(optionNames).toContain("--locale");
    expect(optionNames).toContain("--force");
  });

  test("command description is set", async () => {
    const { fillCommand } = await import("./fill");

    expect(fillCommand.description()).toBe("Fill missing translations using AI");
  });
});
