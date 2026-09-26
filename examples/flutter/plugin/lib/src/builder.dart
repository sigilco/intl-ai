import 'dart:io';

import 'package:build/build.dart';
import 'package:path/path.dart' as p;

import 'options.dart';

/// Factory used by `build_runner` to create the builder instance.
Builder intlAiBuilder(BuilderOptions options) {
  return IntlAiBuilder(
    IntlAiBuilderOptions.fromBuilderOptions(options.config),
  );
}

/// A `build_runner` builder that invokes the `intl-ai` CLI to translate locale
/// files at build time. It has zero runtime impact — translations are written
/// to disk before the Flutter app is bundled.
class IntlAiBuilder implements Builder {
  final IntlAiBuilderOptions options;

  IntlAiBuilder(this.options);

  @override
  Map<String, List<String>> get buildExtensions => {
        r'$package$': ['intl-ai.done'],
      };

  @override
  Future<void> build(BuildStep buildStep) async {
    final packageDir = p.dirname(buildStep.inputId.path);
    final configPath = p.join(packageDir, options.configPath);

    final args = ['fill', '--config', configPath];

    if (options.verbose) {
      log.info('[intl-ai] running: ${options.executable} ${args.join(' ')}');
    }

    final result = await Process.run(
      options.executable,
      args,
      workingDirectory: packageDir,
      runInShell: true,
      stdoutEncoding: utf8,
      stderrEncoding: utf8,
    );

    if (options.verbose) {
      if (result.stdout.isNotEmpty) {
        log.info(result.stdout);
      }
      if (result.stderr.isNotEmpty) {
        log.warning(result.stderr);
      }
    }

    if (result.exitCode != 0) {
      throw Exception(
        'intl-ai fill failed (exit ${result.exitCode}): ${result.stderr}',
      );
    }

    await buildStep.writeAsString(
      AssetId(buildStep.inputId.package, 'intl-ai.done'),
      'Translations generated at ${DateTime.now().toIso8601String()}\n',
    );
  }
}
