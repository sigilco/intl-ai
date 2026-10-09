---
title: Android Jetpack
description: Traducción de IA para aplicaciones Android a través del paquete intlai-android o la CLI intl-ai.
---

# Android Jetpack

Dos formas de integrar `intl-ai` en un proyecto de Android Jetpack:

- El paquete `intlai-android` integra el motor de traducción en su aplicación a través de enlaces UniFFI, por lo que `fill`, `check` y `status` se ejecutan en el dispositivo sin necesidad de incluir la CLI.
- Una tarea de Gradle que ejecuta la CLI `intl-ai` traduce los archivos de configuración de idioma en tiempo de compilación y los incluye como assets, sin sobrecarga en tiempo de ejecución.

## El paquete intlai-android

El AAR `io.github.sigilco:intlai-android` incluye los enlaces UniFFI de Kotlin más `libintl_ai_uniffi.so` para `arm64-v8a`, `armeabi-v7a`, `x86` y `x86_64`. Envuelve la API `uniffi.intl_ai_uniffi`: un objeto `IntlAi` con `fill`, `check`, `status` y variantes `*Json`.

```kotlin
dependencies {
    implementation("io.github.sigilco:intlai-android:0.7.0")
}
```

::: warning Calls are synchronous
Cada método de `IntlAi` se bloquea en operaciones de E/S de configuración y llamadas HTTP al proveedor. Siempre despache a `Dispatchers.IO` o a otro hilo en segundo plano; nunca llame a `IntlAi` en el hilo principal.
:::

### Llamar al motor

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

`IntlAi` lee el mismo `intl-ai.toml` que usa la CLI. `fill` escribe los archivos de configuración de idioma traducidos y fragmentos de lockfile, por lo que `locale_dir` debe apuntar a un directorio escribible como `filesDir`; copie la configuración y las configuraciones de idioma de origen allí en el primer lanzamiento. Para ejecuciones de solo lectura de `check` y `status`, los archivos en assets también funcionan. Cuando no incluye un archivo de configuración, `IntlAi.fromConfigString(config, ConfigFormat.TOML, workingDir)` construye una de forma inline.

### Compilar el paquete

El módulo reside en `kotlin/intlai-android` y compila la cdylib de Rust para las cuatro ABI con `cargo-ndk`:

```sh
./gradlew :intlai-android:assembleRelease   # build/outputs/aar/intlai-android-release.aar
./gradlew publishToMavenLocal               # io.github.sigilco:intlai-android:0.7.0
```

Consulte `kotlin/README.md` para conocer los requisitos del SDK/NDK de Android y la variante de JVM de escritorio de `intlai`.

## Alternativa: ejecutar la CLI en tiempo de compilación

Almacene los archivos de configuración de idioma de origen en `src/main/assets/locales/`, ejecute `intl-ai fill` como una tarea de Gradle y cargue el JSON desde assets en tiempo de ejecución:

```
app/
├── build.gradle.kts
├── intl-ai.toml
└── src/main/assets/locales/
    ├── en.json
    └── es.json
```

En `app/build.gradle.kts`, registre una tarea que invoque la CLI antes de que se fusionen los recursos:

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

Para Windows, la tarea recurre a `cmd /c` para que el binario pueda encontrarse en `PATH`. Esta ruta necesita que `intl-ai` esté instalado en su `PATH` (consulte [Installation](/es/guide/getting-started#installation)) y `intl-ai.toml` en `app/` con `locale_dir` apuntando a `src/main/assets/locales`.

## Cargar traducciones en tiempo de ejecución

Lea los archivos JSON desde assets y analícelos con la biblioteca JSON de su elección:

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

`intl-ai` solo escribe traducciones; no impone una API de tiempo de ejecución para leerlas.
