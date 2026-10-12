---
title: Localizar documentos
description: Traduce árboles de documentos Markdown y MDX con intl-ai-docs. Experimental.
---

# Localizar documentos

`intl-ai-docs` es un segundo binario que se incluye con cada instalación de `intl-ai`. Traduce los árboles de documentos completos (markdown y MDX) de la misma manera que `intl-ai` traduce los archivos de configuración regional: un árbol de origen en un idioma, un árbol generado por cada configuración regional de destino y un archivo de bloqueo que registra de dónde proviene cada cosa.

::: warning Experimental
El soporte para documentos es opcional. Cada comando `intl-ai-docs` se rechaza a ejecutarse a menos que tu configuración establezca `experimental = true`. El lector MDX maneja las construcciones que utilizan los sitios de documentación reales (frontmatter, cercas de código, callouts, bloques JSX), pero el formato es nuevo: ejecuta `intl-ai-docs check` en CI antes de publicar.
:::

## Instalar

Ya lo tienes. El script de instalación, la fórmula de Homebrew, el paquete npm y mise instalan tanto `intl-ai` como `intl-ai-docs`. Verifica con `intl-ai-docs --help`.

## Cómo funciona

En lugar de claves y valores, `intl-ai-docs` divide cada documento en un **esqueleto** y **segmentos**:

1. **Extracción**: el analizador MDX recorre el documento y emite un esqueleto (la estructura del documento con IDs de marcador de posición) más un segmento por cada tramo traducible: encabezados, párrafos, elementos de lista, celdas de tabla y cadenas de frontmatter.
2. **Rellenar**: los segmentos se traducen en lotes por tu proveedor, exactamente como `intl-ai fill` gestiona las claves del catálogo.
3. **Combinar**: los segmentos traducidos se vuelven a insertar en el esqueleto por rango de bytes. La estructura, los comentarios, las vallas de código, el código en línea y los componentes MDX se conservan byte a byte porque nunca estuvieron dentro de un segmento.

Las regiones protegidas (bloques de código, código en línea, comandos, URLs) no pueden ser corrompidas por construcción: el modelo nunca las ve como texto traducible, y la puerta de escritura verifica la paridad antes de que se escriba algo.

## Inicio rápido

Crea `intl-ai-docs.toml` al lado de tu árbol de documentación:

```toml
experimental = true
layout = "locale-dirs"      # source at docs/en, targets at docs/<locale>
source_dir = "docs"
source = "en"
targets = ["es", "pt"]
batch_size = 8

[provider]
kind = "http"
provider = "openai"
base_url = "${env:AI_BASE_URL}"
model = "${env:AI_MODEL_ID}"
api_key = "${env:AI_API_KEY}"
```

Luego completa y verifica:

```bash
intl-ai-docs fill     # writes docs/es/** and docs/pt/**
intl-ai-docs check    # read-only report; --fail-on invalid for CI
```

Los documentos traducidos se almacenan en `docs/<locale>/` reflejando el árbol fuente. La procedencia se guarda en fragmentos de `intl-ai-docs.lock.d/<locale>/<doc>.toml` con los mismos campos que el archivo de bloqueo del corpus (`origin`, `sourceHash`, `reviewed`, `quality`). Confirma tanto los árboles como los fragmentos.

## Diseños

`layout` controla dónde vive el árbol de origen respecto a sus destinos:

- `locale-dirs`: cada locale es un directorio hermano, incluido el origen (`docs/en`, `docs/es`, `docs/pt`). Esto coincide con docmd i18n y la mayoría de los frameworks de documentación que quieren un directorio por locale.
- `flat`: los archivos fuente se sitúan directamente bajo `source_dir`, y los árboles de destino se escriben en `source_dir/<locale>`. Usa esto cuando tus archivos fuente no pueden vivir en un directorio de locale.

Los enlaces se reescriben por defecto según el idioma: los destinos con raíz `/`-rooted obtienen el prefijo del destino (`/es/guide/...`) para que las páginas traducidas enlacen dentro de su propio árbol. Ajusta o desactiva con `[links]`.

## Lo que permanece en inglés

- Cercas de código, código en línea, comandos, banderas, rutas de archivos y URLs.
- Los componentes MDX/JSX y la sintaxis de llamada acotada; solo se traducen los hijos en prosa.
- Claves de frontmatter distintas de las listadas en `frontmatter.keys` (por defecto `title`, `description`).
- Etiquetas de la barra lateral: nav vive en la configuración del sitio, no en los documentos. Para docmd, coloque un `navigation.json` en cada directorio de configuración regional.

Agregue una entrada `[locale_instructions]` por configuración regional para dirigir la terminología, el dialecto y el registro; la regla de resolución es la misma que la de `intl-ai` (configuración regional exacta, subtag de idioma, luego `*`).

::: tip Keep backticks verbatim
Los modelos a veces traducen dentro del código en línea aunque no deberían. Déjalo claro en `locale_instructions`, por ejemplo: `Never translate text inside backticks; keep code blocks, commands, and identifiers verbatim.`
:::

Consulta [Configuración](/es/guide/documents/configuration/) para obtener la referencia completa de `intl-ai-docs.toml`.
