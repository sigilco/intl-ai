/// Configuration options passed from `build.yaml` to the builder.
class IntlAiBuilderOptions {
  /// Path to the intl-ai JSON config file, relative to the package root.
  final String configPath;

  /// Executable used to invoke the CLI. Defaults to `intl-ai`.
  final String executable;

  /// When `true`, the builder logs the CLI stdout/stderr to stdout.
  final bool verbose;

  const IntlAiBuilderOptions({
    this.configPath = 'intl-ai.config.json',
    this.executable = 'intl-ai',
    this.verbose = false,
  });

  factory IntlAiBuilderOptions.fromBuilderOptions(dynamic options) {
    final map = options is Map<String, dynamic> ? options : <String, dynamic>{};
    return IntlAiBuilderOptions(
      configPath: (map['configPath'] as String?) ?? 'intl-ai.config.json',
      executable: (map['executable'] as String?) ?? 'intl-ai',
      verbose: (map['verbose'] as bool?) ?? false,
    );
  }
}
