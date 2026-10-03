---
title: Android Jetpack
description: AI translation for Android apps through the intlai-android package or the intl-ai CLI.
---

# Android Jetpack

Two ways to integrate `intl-ai` into an Android Jetpack project:

- The `intlai-android` package embeds the translation engine in your app through UniFFI bindings, so `fill`, `check`, and `status` run on device without shipping the CLI.
- A Gradle task that runs the `intl-ai` CLI translates locale files at build time and ships them as assets, with zero runtime overhead.

## The intlai-android package

The `io.github.sigilco:intlai-android` AAR bundles the Kotlin UniFFI bindings plus `libintl_ai_uniffi.so` for `arm64-v8a`, `armeabi-v7a`, `x86`, and `x86_64`. It wraps the `uniffi.intl_ai_uniffi` API: an `IntlAi` object with `fill`, `check`, `status`, and `*Json` variants.

```kotlin
dependencies {
    implementation("io.github.sigilco:intlai-android:0.7.0")
}
```

::: warning Calls are synchronous
Every `IntlAi` method blocks on config I/O and provider HTTP calls. Always dispatch to `Dispatchers.IO` or another background thread; never call `IntlAi` on the main thread.
:::

### Call the engine

```kotlin
import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.intl_ai_uniffi.FillOptions
import uniffi.intl_ai_uniffi.IntlAi

suspend fun translate(context: Context) = withContext(Dispatchers.IO) {
    IntlAi(
        configPath = "${context.filesDir}/intl-ai.toml",
        workingDir = context.filesDir.absolutePath,
    ).use { api ->
        api.check(checkOptions)
        api.fill(fillOptions)
    }
}
```

`IntlAi` reads the same `intl-ai.toml` the CLI uses. `fill` writes translated locale files and lockfile shards, so `locale_dir` must point to a writable directory such as `filesDir`; copy the config and source locales there on first launch. For read-only `check` and `status` runs, files under assets work too. When you do not ship a config file, `IntlAi.fromConfigString(config, ConfigFormat.TOML, workingDir)` builds one inline.

### Build the package

The module lives in `kotlin/intlai-android` and compiles the Rust cdylib for the four ABIs with `cargo-ndk`:

```sh
./gradlew :intlai-android:assembleRelease   # build/outputs/aar/intlai-android-release.aar
./gradlew publishToMavenLocal               # io.github.sigilco:intlai-android:0.7.0
```

See `kotlin/README.md` for the Android SDK/NDK requirements and the `intlai` desktop JVM variant.

## Alternative: run the CLI at build time

Store source locale files in `src/main/assets/locales/`, run `intl-ai fill` as a Gradle task, and load the JSON from assets at runtime:

```
app/
├── build.gradle.kts
├── intl-ai.toml
└── src/main/assets/locales/
    ├── en.json
    └── es.json
```

In `app/build.gradle.kts`, register a task that invokes the CLI before resources are merged:

```kotlin
import java.io.ByteArrayOutputStream

plugins {
    alias(libs.plugins.android.application)
}

android { /* ... */ }

tasks.register<Exec>("intlAiFill") {
    group = "intl-ai"
    description = "Translate missing locale keys with intl-ai"

    commandLine("intl-ai", "fill", "--config", "${projectDir}/intl-ai.toml")

    // Only run when source locale files change.
    inputs.dir("${projectDir}/src/main/assets/locales")
    outputs.dir("${projectDir}/src/main/assets/locales")

    doFirst {
        if (org.gradle.internal.os.OperatingSystem.current().isWindows) {
            commandLine("cmd", "/c", "intl-ai", "fill", "--config", "${projectDir}/intl-ai.toml")
        }
    }
}

tasks.named("preBuild").configure {
    dependsOn("intlAiFill")
}
```

For Windows, the task falls back to `cmd /c` so the binary can be found on `PATH`. This path needs `intl-ai` installed on your `PATH` (see [Installation](/guide/getting-started#installation)) and `intl-ai.toml` in `app/` with `locale_dir` pointing at `src/main/assets/locales`.

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

`intl-ai` only writes translations; it does not impose a runtime API for reading them.
