import { describe, test, expect, beforeEach, afterEach } from "vitest";
import { findMissingTranslations, LockfileManager, readJsonFile, writeJsonFile } from "./index";
import { createTempDir, cleanupTempDir, createLocaleFile } from "./testing/fixtures";
import { writeFileSync, mkdirSync, existsSync } from "fs";
import { join } from "path";

describe("Cross-package integration", () => {
  let tempDir: string;
  let originalCwd: string;

  beforeEach(() => {
    tempDir = createTempDir("integration-test-");
    originalCwd = process.cwd();
    process.chdir(tempDir);
  });

  afterEach(() => {
    process.chdir(originalCwd);
    cleanupTempDir(tempDir);
  });

  describe("Core functions work together", () => {
    test("findMissingTranslations and LockfileManager integrate correctly", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const sourceLocale = {
        greeting: "Hello",
        farewell: "Goodbye",
        nested: {
          message: "Welcome",
        },
      };
      createLocaleFile(localeDir, "en", sourceLocale);

      const targetLocale = {
        greeting: "Bonjour",
      };
      createLocaleFile(localeDir, "fr", targetLocale);

      const lockfileManager = new LockfileManager(localeDir);
      const lockfileEntries = new Map<string, { sourceHash: string }>();

      for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
        lockfileEntries.set(key, { sourceHash: entry.sourceHash });
      }

      const diff = findMissingTranslations({
        sourceLocale: readJsonFile(join(localeDir, "en.json")),
        targetLocale: readJsonFile(join(localeDir, "fr.json")),
        locale: "fr",
        lockfileEntries,
      });

      expect(diff.missing.length).toBe(2);
      expect(diff.missing[0].key).toBe("farewell");
      expect(diff.missing[1].key).toBe("nested.message");

      const sourceValue1 = diff.missing[0].source;
      const sourceValue2 = diff.missing[1].source;

      lockfileManager.setEntry("farewell", "fr", {
        key: "farewell",
        locale: "fr",
        sourceHash: lockfileManager.hashSource(sourceValue1),
        translated: "Au revoir",
        origin: "ai",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });

      lockfileManager.setEntry("nested.message", "fr", {
        key: "nested.message",
        locale: "fr",
        sourceHash: lockfileManager.hashSource(sourceValue2),
        translated: "Bienvenue",
        origin: "ai",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });

      lockfileManager.save();

      const lockfilePath = join(localeDir, "intl-ai.lock.json");
      expect(existsSync(lockfilePath)).toBe(true);

      const lockfileData = readJsonFile(lockfilePath);
      expect(lockfileData.version).toBe(1);
      expect(lockfileData.entries["fr:farewell"]).toBeDefined();
      expect(lockfileData.entries["fr:nested.message"]).toBeDefined();

      const entry1 = lockfileData.entries["fr:farewell"];
      expect(entry1.translated).toBe("Au revoir");
      expect(entry1.origin).toBe("ai");

      const entry2 = lockfileData.entries["fr:nested.message"];
      expect(entry2.translated).toBe("Bienvenue");
      expect(entry2.origin).toBe("ai");
    });
  });

  describe("CLI + Core integration", () => {
    test("CLI workflow pattern: findMissingTranslations and LockfileManager", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const sourceLocale = {
        welcome: "Welcome",
        goodbye: "Goodbye",
      };
      createLocaleFile(localeDir, "en", sourceLocale);

      const targetLocale = {
        welcome: "Bienvenido",
      };
      createLocaleFile(localeDir, "es", targetLocale);

      const lockfileManager = new LockfileManager(localeDir);
      const lockfileEntries = new Map<string, { sourceHash: string }>();
      for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
        const [locale] = key.split(":");
        if (locale === "es") {
          lockfileEntries.set(key, { sourceHash: entry.sourceHash });
        }
      }

      const diff = findMissingTranslations({
        sourceLocale: readJsonFile(join(localeDir, "en.json")),
        targetLocale: readJsonFile(join(localeDir, "es.json")),
        locale: "es",
        lockfileEntries,
      });

      expect(diff.missing.length).toBe(1);
      expect(diff.missing[0].key).toBe("goodbye");
      expect(diff.missing[0].source).toBe("Goodbye");

      const sourceValue = diff.missing[0].source;
      lockfileManager.setEntry("goodbye", "es", {
        key: "goodbye",
        locale: "es",
        sourceHash: lockfileManager.hashSource(sourceValue),
        translated: "Adiós",
        origin: "ai",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });

      lockfileManager.save();

      const lockfilePath = join(localeDir, "intl-ai.lock.json");
      expect(existsSync(lockfilePath)).toBe(true);
      const lockfileData = readJsonFile(lockfilePath);
      expect(lockfileData.entries["es:goodbye"]).toBeDefined();
      expect(lockfileData.entries["es:goodbye"].translated).toBe("Adiós");
    });
  });

  describe("Vite plugin + Core integration", () => {
    test("Vite plugin workflow: findMissingTranslations, update files, LockfileManager", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const sourceLocale = {
        hello: "Hello",
        thanks: "Thank you",
      };
      createLocaleFile(localeDir, "en", sourceLocale);

      const targetLocale = {
        hello: "Hallo",
      };
      createLocaleFile(localeDir, "de", targetLocale);

      const lockfileManager = new LockfileManager(localeDir);
      const lockfileEntries = new Map<string, { sourceHash: string }>();
      for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
        lockfileEntries.set(key, { sourceHash: entry.sourceHash });
      }

      const locale = "de";
      const sourcePath = join(localeDir, "en.json");
      const targetPath = join(localeDir, "de.json");

      const diff = findMissingTranslations({
        sourceLocale: readJsonFile(sourcePath),
        targetLocale: readJsonFile(targetPath),
        locale,
        lockfileEntries,
      });

      expect(diff.missing.length).toBe(1);
      expect(diff.missing[0].key).toBe("thanks");

      const sourceValue = diff.missing[0].source;

      const targetLocaleData = readJsonFile(targetPath);
      targetLocaleData["thanks"] = "Danke";

      lockfileManager.setEntry("thanks", locale, {
        key: "thanks",
        locale,
        sourceHash: lockfileManager.hashSource(sourceValue),
        translated: "Danke",
        origin: "ai",
        timestamp: new Date().toISOString(),
      });

      writeJsonFile(targetPath, targetLocaleData);
      lockfileManager.save();

      const updatedTargetLocale = readJsonFile(targetPath);
      expect(updatedTargetLocale.thanks).toBe("Danke");

      const lockfilePath = join(localeDir, "intl-ai.lock.json");
      expect(existsSync(lockfilePath)).toBe(true);
      const lockfileData = readJsonFile(lockfilePath);
      expect(lockfileData.entries["de:thanks"]).toBeDefined();
    });
  });

  describe("Next.js wrapper + Core integration", () => {
    test("Next.js wrapper workflow: findMissingTranslations with lockfile", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const sourceLocale = {
        greeting: "Hello",
        success: "Success",
      };
      createLocaleFile(localeDir, "en", sourceLocale);

      const targetLocale = {
        greeting: "Bonjour",
      };
      createLocaleFile(localeDir, "fr", targetLocale);

      const lockfileManager = new LockfileManager(localeDir);
      const lockfileEntries = new Map(
        Object.entries(lockfileManager.getAllEntries()).map(([key, entry]) => [
          key,
          { sourceHash: entry.sourceHash },
        ]),
      );

      const sourceLocaleData = readJsonFile(join(localeDir, "en.json"));
      const targetLocaleData = readJsonFile(join(localeDir, "fr.json"));

      const diff = findMissingTranslations({
        sourceLocale: sourceLocaleData,
        targetLocale: targetLocaleData,
        locale: "fr",
        lockfileEntries,
      });

      expect(diff.missing.length).toBe(1);
      expect(diff.missing[0].key).toBe("success");

      const sourceValue = diff.missing[0].source;

      lockfileManager.setEntry("success", "fr", {
        key: "success",
        locale: "fr",
        sourceHash: lockfileManager.hashSource(sourceValue),
        translated: "Succès",
        origin: "ai",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });

      lockfileManager.save();

      const lockfilePath = join(localeDir, "intl-ai.lock.json");
      expect(existsSync(lockfilePath)).toBe(true);
      const lockfileData = readJsonFile(lockfilePath);
      expect(lockfileData.entries["fr:success"]).toBeDefined();
      expect(lockfileData.entries["fr:success"].translated).toBe("Succès");
    });
  });

  describe("End-to-end translation flow", () => {
    test("complete workflow: create locale files, run translation, verify results", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const englishLocale = {
        welcome: "Welcome",
        hello: "Hello",
        goodbye: "Goodbye",
        thanks: "Thank you",
        nested: {
          title: "Title",
        },
      };
      createLocaleFile(localeDir, "en", englishLocale);

      const spanishLocale = {
        welcome: "Bienvenido",
        goodbye: "Adiós",
        nested: {
          title: "Título",
        },
      };
      createLocaleFile(localeDir, "es", spanishLocale);

      createLocaleFile(localeDir, "fr", {});

      for (const locale of ["es", "fr"]) {
        const lockfileManager = new LockfileManager(localeDir);
        const lockfileEntries = new Map<string, { sourceHash: string }>();

        for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
          const [entryLocale] = key.split(":");
          if (entryLocale === locale) {
            lockfileEntries.set(key, { sourceHash: entry.sourceHash });
          }
        }

        const diff = findMissingTranslations({
          sourceLocale: readJsonFile(join(localeDir, "en.json")),
          targetLocale: readJsonFile(join(localeDir, `${locale}.json`)),
          locale,
          lockfileEntries,
        });

        if (diff.missing.length > 0) {
          const targetLocaleData = readJsonFile(join(localeDir, `${locale}.json`));
          for (const missing of diff.missing) {
            const parts = missing.key.split(".");
            let current: any = targetLocaleData;
            for (let i = 0; i < parts.length - 1; i++) {
              if (!(parts[i] in current)) {
                current[parts[i]] = {};
              }
              current = current[parts[i]];
            }

            const translatedMap: Record<string, Record<string, string>> = {
              es: {
                Hello: "Hola",
                "Thank you": "Gracias",
                Welcome: "Bienvenido",
                Goodbye: "Adiós",
              },
              fr: {
                Hello: "Bonjour",
                "Thank you": "Merci",
                Welcome: "Bienvenue",
                Goodbye: "Au revoir",
              },
            };
            const translated = translatedMap[locale]?.[missing.source] || missing.source;

            current[parts[parts.length - 1]] = translated;
            lockfileManager.setEntry(missing.key, locale, {
              key: missing.key,
              locale,
              sourceHash: lockfileManager.hashSource(missing.source),
              translated,
              origin: "ai",
              model: "mock-model",
              timestamp: new Date().toISOString(),
            });
          }

          writeJsonFile(join(localeDir, `${locale}.json`), targetLocaleData);
          lockfileManager.save();
        }
      }

      const spanishResult = readJsonFile(join(localeDir, "es.json"));
      expect(spanishResult.hello).toBe("Hola");
      expect(spanishResult.thanks).toBe("Gracias");
      expect(spanishResult.nested.title).toBe("Título");

      const frenchResult = readJsonFile(join(localeDir, "fr.json"));
      expect(frenchResult.welcome).toBe("Bienvenue");
      expect(frenchResult.hello).toBe("Bonjour");
      expect(frenchResult.goodbye).toBe("Au revoir");
      expect(frenchResult.thanks).toBe("Merci");

      const lockfileManager = new LockfileManager(localeDir);
      const allEntries = lockfileManager.getAllEntries();

      expect(allEntries["es:hello"]).toBeDefined();
      expect(allEntries["es:thanks"]).toBeDefined();

      expect(allEntries["fr:welcome"]).toBeDefined();
      expect(allEntries["fr:hello"]).toBeDefined();
      expect(allEntries["fr:goodbye"]).toBeDefined();
      expect(allEntries["fr:thanks"]).toBeDefined();

      const spanishHello = allEntries["es:hello"];
      expect(spanishHello.origin).toBe("ai");
      expect(spanishHello.translated).toBe("Hola");
      expect(spanishHello.sourceHash).toBeDefined();
      expect(spanishHello.timestamp).toBeDefined();
    });

    test("end-to-end flow respects human-edited entries", () => {
      const localeDir = join(tempDir, "locales");
      mkdirSync(localeDir, { recursive: true });

      const englishLocale = {
        custom: "Custom message",
        standard: "Standard message",
      };
      createLocaleFile(localeDir, "en", englishLocale);

      const frenchLocale = {
        custom: "Message personnalisé",
      };
      createLocaleFile(localeDir, "fr", frenchLocale);

      const lockfileManager = new LockfileManager(localeDir);
      lockfileManager.setEntry("custom", "fr", {
        key: "custom",
        locale: "fr",
        sourceHash: lockfileManager.hashSource("Custom message"),
        translated: "Message personnalisé",
        origin: "human",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });
      lockfileManager.save();

      const lockfileEntries = new Map<string, { sourceHash: string }>();
      for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
        lockfileEntries.set(key, { sourceHash: entry.sourceHash });
      }

      const diff = findMissingTranslations({
        sourceLocale: readJsonFile(join(localeDir, "en.json")),
        targetLocale: readJsonFile(join(localeDir, "fr.json")),
        locale: "fr",
        lockfileEntries,
      });

      expect(diff.missing.length).toBe(1);
      expect(diff.missing[0].key).toBe("standard");

      const targetLocaleData = readJsonFile(join(localeDir, "fr.json"));
      targetLocaleData["standard"] = "Message standard";

      lockfileManager.setEntry("standard", "fr", {
        key: "standard",
        locale: "fr",
        sourceHash: lockfileManager.hashSource("Standard message"),
        translated: "Message standard",
        origin: "ai",
        model: "mock-model",
        timestamp: new Date().toISOString(),
      });

      writeJsonFile(join(localeDir, "fr.json"), targetLocaleData);
      lockfileManager.save();

      const finalFrench = readJsonFile(join(localeDir, "fr.json"));
      expect(finalFrench.custom).toBe("Message personnalisé");
      expect(finalFrench.standard).toBe("Message standard");

      const allEntries = lockfileManager.getAllEntries();
      expect(allEntries["fr:custom"].origin).toBe("human");
      expect(allEntries["fr:standard"].origin).toBe("ai");
    });
  });
});
