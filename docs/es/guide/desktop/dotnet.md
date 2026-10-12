---
title: .NET / C#
description: Traducción con IA en tiempo de compilación para i18n de .NET mediante MSBuild. Traducciones en tiempo de compilación, cero en tiempo de ejecución.
---

# .NET / C#

Puede integrar `intl-ai` en un proyecto .NET agregando un destino de MSBuild que ejecute la CLI antes de la compilación. Todas las traducciones ocurren en tiempo de compilación, por lo que no hay sobrecarga en tiempo de ejecución.

## Disposición del proyecto

```
MyApp/
├── MyApp.csproj
├── intl-ai.toml
└── Resources/
    ├── en.json
    └── es.json
```

Almacene los archivos de configuración regional de origen en `Resources/`, ejecute `intl-ai fill` como un destino de MSBuild e incruste o copie los archivos en el directorio de salida.

## Agregue un destino de MSBuild

Agregue el siguiente destino a su archivo `.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <None Update="Resources\*.json">
      <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
    </None>
  </ItemGroup>

  <Target Name="IntlAiFill" BeforeTargets="BeforeBuild">
    <Exec Command="intl-ai fill --config $(MSBuildProjectDirectory)/intl-ai.toml" />
  </Target>

</Project>
```

El destino `IntlAiFill` se ejecuta antes de cada compilación, invocando la CLI para rellenar las traducciones faltantes. Debido a que los elementos `Resources\*.json` usan `CopyToOutputDirectory=PreserveNewest`, los archivos generados se copian automáticamente a la carpeta de salida.

## Cargue las traducciones en tiempo de ejecución

Utilice `System.Text.Json` para cargar los archivos de configuración regional desde el directorio de salida:

```csharp
using System.Text.Json;

public record LocaleMessages(string Hello, string Goodbye);

public static class Translations
{
    public static LocaleMessages Load(string locale)
    {
        var path = Path.Combine(AppContext.BaseDirectory, "Resources", $"{locale}.json");
        var json = File.ReadAllText(path);
        return JsonSerializer.Deserialize<LocaleMessages>(json)!;
    }
}
```

## Requisitos

- `intl-ai` instalado en su `PATH` (see [Installation](/es/guide/getting-started#installation)).
- `intl-ai.toml` junto a su archivo `.csproj`. Ajuste `locale_dir` para que apunte a `Resources`.

## Ejemplo

Utilice la disposición del proyecto anterior y adapte la a su propia aplicación WPF, WinUI, ASP.NET o de consola. `intl-ai` solo escribe traducciones; no impone una API en tiempo de ejecución.
