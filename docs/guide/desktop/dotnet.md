# .NET / C#

You can integrate `intl-ai` into a .NET project by adding an MSBuild target that runs the CLI before the build. All translations happen at build time, so there is zero runtime overhead.

## Project layout

```
MyApp/
├── MyApp.csproj
├── intl-ai.config.json
└── Resources/
    ├── en.json
    └── es.json
```

Store source locale files in `Resources/`, run `intl-ai fill` as an MSBuild target, and embed or copy them into the output directory.

## Add an MSBuild target

Add the following target to your `.csproj` file:

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
    <Exec Command="intl-ai fill --config $(MSBuildProjectDirectory)/intl-ai.config.json" />
  </Target>

</Project>
```

The `IntlAiFill` target runs before every build, invoking the CLI to fill missing translations. Because the `Resources\*.json` items use `CopyToOutputDirectory=PreserveNewest`, the generated files are copied to the output folder automatically.

## Load translations at runtime

Use `System.Text.Json` to load locale files from the output directory:

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

## Requirements

- `intl-ai` installed on your `PATH` (see [Installation](/guide/installation)).
- `intl-ai.config.json` next to your `.csproj` file. Adjust `localeDir` to point to `Resources`.

## Example

Use the project layout above and adapt it to your own WPF, WinUI, ASP.NET, or console app. `intl-ai` only writes translations; it does not impose a runtime API.
