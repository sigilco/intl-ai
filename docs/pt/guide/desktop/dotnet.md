---
title: .NET / C#
description: Tradução com IA em tempo de construção para .NET i18n via MSBuild. Traduções em tempo de construção, zero sobrecarga em tempo de execução.
---

# .NET / C#

Pode integrar o `intl-ai` num projeto .NET adicionando um destino MSBuild que executa a CLI antes da construção. Todas as traduções ocorrem em tempo de construção, portanto não há nenhuma sobrecarga em tempo de execução.

## Layout do projeto

```
MyApp/
├── MyApp.csproj
├── intl-ai.toml
└── Resources/
    ├── en.json
    └── es.json
```

Armazene os ficheiros de locale de origem em `Resources/`, execute `intl-ai fill` como um destino MSBuild e incorporá-los ou copiá-los para o diretório de saída.

## Adicionar um destino MSBuild

Adicione o seguinte destino ao ficheiro `.csproj`:

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

O destino `IntlAiFill` executa antes de cada construção, invocando a CLI para preencher as traduções em falta. Como os itens `Resources\*.json` utilizam `CopyToOutputDirectory=PreserveNewest`, os ficheiros gerados são copiados automaticamente para a pasta de saída.

## Carregar traduções em tempo de execução

Use `System.Text.Json` para carregar ficheiros de locale a partir do diretório de saída:

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

- `intl-ai` instalado no seu `PATH` (consulte [Instalação](/pt/guide/getting-started#installation)).
- `intl-ai.toml` ao lado do ficheiro `.csproj`. Ajuste `locale_dir` para apontar para `Resources`.

## Exemplo

Use o layout de projeto acima e adapte-o ao seu próprio WPF, WinUI, ASP.NET ou aplicativo de consola. `intl-ai` apenas escreve traduções; não impõe uma API de tempo de execução.
