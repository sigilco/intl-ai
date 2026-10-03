package io.github.sigilco.intlai

import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.writeText
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import uniffi.intl_ai_uniffi.CheckOptions
import uniffi.intl_ai_uniffi.ConfigFormat
import uniffi.intl_ai_uniffi.FillOptions
import uniffi.intl_ai_uniffi.IntlAi

// Golden-path smoke test of the FFI surface against the replay transport,
// mirroring crates/intl-ai-uniffi/tests/smoke.rs: no real API is touched.
// Exercises the native library bundled in the JAR via JNA.
class IntlAiSmokeTest {

    private val configToml = """
locale_dir = "locales"
source = "en"
targets = ["fr", "de"]

[provider]
kind = "replay"
file = "cassette.json"

[check]
fail_on = ["missing", "stale"]
""".trimIndent()

    private val cassette = """
{
  "fr": {
    "Settings": "Paramètres",
    "Hello": "Bonjour"
  },
  "de": {
    "Settings": "Einstellungen",
    "Hello": "Hallo",
    "Home": "Startseite"
  }
}
""".trimIndent()

    private fun fixture(): Path {
        val dir = Files.createTempDirectory("intlai-test")
        dir.resolve("intl-ai.toml").writeText(configToml)
        dir.resolve("locales").createDirectories()
        dir.resolve("locales/en.json").writeText(
            """
{
  "nav": {
    "home": "Home",
    "settings": "Settings"
  },
  "greeting": "Hello"
}
""".trimIndent()
        )
        dir.resolve("locales/fr.json").writeText(
            """
{
  "nav": {
    "home": "Accueil humain"
  }
}
""".trimIndent()
        )
        dir.resolve("cassette.json").writeText(cassette)
        return dir
    }

    private fun fillOptions() = FillOptions(
        locales = emptyList(),
        keys = emptyList(),
        keysFile = null,
        stale = false,
        regenerate = false,
        includeHuman = false,
        dryRun = false,
        noCache = true,
        validate = emptyList(),
        noValidate = false,
        judgeThreshold = null,
        yes = false,
    )

    private fun checkOptions() = CheckOptions(
        locales = emptyList(),
        keys = emptyList(),
        keysFile = null,
        origin = null,
        failOn = null,
        noCache = true,
    )

    @Test
    fun pathConstructorFillsAndChecks() {
        val dir = fixture()
        IntlAi(dir.resolve("intl-ai.toml").toString(), dir.toString()).use { api ->
            assertEquals("en", api.sourceLocale())
            assertEquals(listOf("fr", "de"), api.targetLocales())

            assertTrue(api.check(checkOptions()).hasIssues)

            val report = api.fill(fillOptions())
            assertTrue(report.failures.isEmpty())
            assertEquals(2uL, report.locales["fr"]!!.written)
            assertEquals(1uL, report.locales["fr"]!!.adoptedHuman)
            assertEquals(3uL, report.locales["de"]!!.written)

            val fr = api.status(emptyList()).first { it.locale == "fr" }
            assertEquals(3uL, fr.entries)
            assertEquals(0uL, fr.missing)
            assertEquals(1uL, fr.human)
        }
    }

    @Test
    fun inlineStringConstructor() {
        val dir = fixture()
        IntlAi.fromConfigString(configToml, ConfigFormat.TOML, dir.toString()).use { api ->
            val status = api.status(listOf("de"))
            assertEquals(3uL, status[0].missing)
        }
    }
}
