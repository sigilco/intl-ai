# Android Jetpack

You can integrate `intl-ai` into an Android Jetpack project by adding a Gradle task that runs the CLI before the build. All translations happen at build time, so there is zero runtime overhead.

## Project layout

```
app/
├── build.gradle.kts
├── intl-ai.config.json
└── src/main/assets/locales/
    ├── en.json
    └── es.json
```

Store source locale files in `src/main/assets/locales/`, run `intl-ai fill` as a Gradle task, and load them from assets at runtime.

## Add a Gradle task

In your `app/build.gradle.kts`, register a task that invokes the CLI before resources are merged:

```kotlin
import java.io.ByteArrayOutputStream

plugins {
    alias(libs.plugins.android.application)
}

android { /* ... */ }

tasks.register<Exec>("intlAiFill") {
    group = "intl-ai"
    description = "Translate missing locale keys with intl-ai"

    commandLine("intl-ai", "fill", "--config", "${projectDir}/intl-ai.config.json")

    // Only run when source locale files change.
    inputs.dir("${projectDir}/src/main/assets/locales")
    outputs.dir("${projectDir}/src/main/assets/locales")

    doFirst {
        if (org.gradle.internal.os.OperatingSystem.current().isWindows) {
            commandLine("cmd", "/c", "intl-ai", "fill", "--config", "${projectDir}/intl-ai.config.json")
        }
    }
}

tasks.named("preBuild").configure {
    dependsOn("intlAiFill")
}
```

For Windows, the task falls back to `cmd /c` so the binary can be found on `PATH`.

## Load translations at runtime

Read JSON files from assets and parse them with your JSON library of choice:

```kotlin
import android.content.Context
import kotlinx.serialization.json.Json
import kotlinx.serialization.Serializable

@Serializable
data class LocaleMessages(
    val hello: String,
    val goodbye: String
)

fun loadLocale(context: Context, locale: String): LocaleMessages {
    val json = context.assets.open("locales/$locale.json").bufferedReader().use { it.readText() }
    return Json.decodeFromString(LocaleMessages.serializer(), json)
}
```

## Requirements

- `intl-ai` installed on your `PATH` (see [Installation](/guide/getting-started#installation)).
- `intl-ai.config.json` in `app/`. Adjust `localeDir` to point to `src/main/assets/locales`.

## Example

Use the project layout above and adapt it to your own Jetpack Compose or View-based app. `intl-ai` only writes translations; it does not impose a runtime API.
