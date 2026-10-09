---
title: Contribuir
description: Contribuye a la documentación de intl-ai. Edita cualquier página de documentación en GitHub y abre una solicitud de extracción.
---

# Contribuyendo a la documentación

Gracias por su interés en mejorar la documentación de intl-ai. Esta guía cubre todo lo que necesita saber sobre cómo contribuir a la documentación.

Para las pautas generales de contribución, los flujos de ramificación, las convenciones de commit y el proceso de desarrollo completo, consulte nuestro [main Contributing Guide](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Configuración de la documentación

La documentación se construye con [docmd](https://docmd.io/), un generador estático de sitios web centrado en Markdown.

### Requisitos previos

- Node.js 22+ y pnpm 11+
- Familiaridad básica con Markdown
- Un editor de texto (se recomienda VS Code)

### Instalación

```bash
pnpm install
```

## Ejecutar la documentación localmente

Inicia el servidor de desarrollo para previsualizar tus cambios en tiempo real:

```bash
pnpm docs:dev
```

Esto inicia un servidor de desarrollo local (típicamente en `http://localhost:3000`) con recarga en vivo para actualizaciones de vista previa instantánea.

## Compilación para producción

```bash
pnpm docs:build
```

Esto genera archivos estáticos optimizados en el directorio `docs/.vitepress/dist/` y copia los activos de `docs/public/` a la raíz del sitio.

## Previsualizar la compilación de producción

Después de construir, sirva el directorio de salida con cualquier servidor de archivos estáticos para verificar la salida de producción.

## Guías de escritura de documentación

### Estructura de archivos

```
docmd.config.js     # site config, navigation, plugins
assets/             # images served at /assets/
docs/
  guide/
    getting-started.md
    ai-model.md
    configuration.md
    api.md
    contributing.md
    build-systems/
    i18n-libraries/
    mobile/
    desktop/
  public/           # files copied to the site root on build
    install.sh
    schema/
  index.md
```

### Convenciones de Markdown

- **Encabezados:** Usa `#` para el título de la página, `##` para secciones, `###` para subsecciones
- **Bloques de código:** Especifica el lenguaje para el resaltado de sintaxis (` ```bash `, ` ```typescript `, etc.)
- **Pestañas:** Use `::: tabs` containers with `== tab "Label"` entries for alternative instructions (for example, comandos del gestor de paquetes)
- **Links:** Use rutas raíz-relative para enlaces internos, por ejemplo `[link text](/es/guide/getting-started/)`
- **Longitud de línea:** Mantén las líneas bajo 100 caracteres

### Frontmatter

Cada página de documentación debería incluir el frontmatter YAML en la parte superior:

```yaml
---
title: Page title
description: One sentence for SEO and search results.
---
```

### Enlaces internos

```markdown
[Getting started guide](/guide/getting-started/)
```

### Enlaces externos

Los enlaces externos se abren automáticamente en una nueva pestaña.

## Agregar nuevas páginas de guía

1. Crear un nuevo archivo `.md` en `docs/guide/` (o el subdirectorio correspondiente).
2. Agregar el frontmatter YAML requerido con un campo `title`.
3. Agrega la página al arreglo `navigation` en `docmd.config.js`.
4. Ejecute `pnpm docs:dev` y verifique que su página aparezca y se represente correctamente.
5. Enviar una solicitud de pull request siguiendo la [Guía de Contribución principal](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Referencia de comandos de documentación

| Comando              | Propósito                                         |
| -------------------- | ----------------------------------------------- |
| `pnpm docs:dev`      | Iniciar el servidor de desarrollo local con recarga en vivo |
| `pnpm docs:build`    | Construir archivos estáticos listos para producción             |
| `pnpm docs:validate` | Verificar todos los enlaces internos en busca de objetivos rotos     |

## Despliegue

La documentación se despliega en Cloudflare Pages en `https://intl-ai.illo.fyi/` cuando los cambios se fusionan a la rama `main`.

El proceso de despliegue:

1. Detecta cambios en el directorio `docs/`
2. Ejecuta `pnpm docs:build` para generar archivos estáticos
3. Despliega la salida a Cloudflare Pages

No se requieren pasos de despliegue manual.

## Antes de enviar cambios de documentación

- Ejecuta `pnpm docs:dev` y verifica que todas las páginas se rendericen correctamente
- Ejecutar `pnpm docs:validate` para comprobar los enlaces rotos
- Comprobar que todos los ejemplos de código estén completos y probados
- Sigue las reglas de estilo de escritura en `AGENTS.md` (mayúscula de oración, sin guiones largos, sin emoji)

## Código de conducta

Por favor lea y siga nuestro [Código de Conducta](https://github.com/sigilco/intl-ai/blob/main/CODE_OF_CONDUCT.md).

## ¿Preguntas o problemas?

- [Abrir un problema](https://github.com/sigilco/intl-ai/issues) para errores o sugerencias de documentación
- [Iniciar una discusión](https://github.com/sigilco/intl-ai/discussions) para preguntas
