---
title: Android Jetpack
description: Tradução de IA para aplicações Android através do pacote intlai-android ou da CLI intl-ai.
---

# Android Jetpack

Duas formas de integrar `intl-ai` num projeto Android Jetpack:

- O pacote `intlai-android` incorpora o motor de tradução na sua aplicação através de ligações UniFFI, por isso `fill`, `check` e `status` correm no dispositivo sem enviar a CLI.
- Uma tarefa Gradle que executa a CLI `intl-ai` traduz ficheiros de localização em tempo de compilação e envia-os como recursos, com zero sobrecarga em tempo de execução.

## O pacote intlai-android

O AAR `io.github.sigilco:intlai-android` agrega as ligações UniFFI Kotlin mais `libintl_ai_uniffi.so` para `arm64-v8a`, `armeabi-v7a`, `x86` e `x86_64`. Envolve a API `uniffi.intl_ai_uniffi`: um objeto `IntlAi` com `fill`, `check`, `status` e variantes `*Json`.

```kotlin
dependencies {
    implementation("io.github.sigilco:intlai-android:0.7.0")
}
```

::: warning Calls are synchronous
Todo método `IntlAi` bloqueia na I/O de configuração e nas chamadas HTTP do fornecedor. Sempre despacho para `Dispatchers.IO` ou outra thread em segundo plano; nunca chame `IntlAi` na thread principal.
:::

### Chamar o motor

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

`IntlAi` lê o mesmo `intl-ai.toml` que a CLI utiliza. `fill` escreve ficheiros de localização traduzidos e fragmentos de lockfile, por isso `locale_dir` deve apontar para um diretório gravável como `filesDir`; copie a configuração e as localizações de origem para lá no primeiro arranque. Para execuções de `check` e `status` apenas leitura, os ficheiros sob assets também funcionam. Quando não envia um ficheiro de configuração, `IntlAi.fromConfigString(config, ConfigFormat.TOML, workingDir)` cria uma inline.

### Construir o pacote

O módulo vive em `kotlin/intlai-android` e compila o cdylib Rust para as quatro ABIs com `cargo-ndk`:

```sh
./gradlew :intlai-android:assembleRelease   # build/outputs/aar/intlai-android-release.aar
./gradlew publishToMavenLocal               # io.github.sigilco:intlai-android:0.7.0
```

Consulte `kotlin/README.md` para os requisitos do SDK/NDK do Android e a variante desktop JVM `intlai`.

## Alternativa: executar a CLI em tempo de compilação

Armazene os ficheiros de localização de origem em `src/main/assets/locales/`, execute `intl-ai fill` como uma tarefa Gradle e carregue o JSON dos recursos em tempo de execução:

```
app/
├── build.gradle.kts
├── intl-ai.toml
└── src/main/assets/locales/
    ├── en.json
    └── es.json
```

Em `app/build.gradle.kts`, registe uma tarefa que invoca a CLI antes de os recursos serem mesclados:

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

Para Windows, a tarefa volta a `cmd /c` para que o binário possa ser encontrado no `PATH`. Este caminho requer que `intl-ai` este instalado no seu `PATH` (ver [Installation](/pt/guide/getting-started#installation)) e `intl-ai.toml` em `app/` com `locale_dir` apontando para `src/main/assets/locales`.

## Carregar traduções em tempo de execução

Leia ficheiros JSON de assets e analise-os com a sua biblioteca JSON preferida:

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

`intl-ai` apenas escreve traduções; não impõe uma API de leitura em tempo de execução.
