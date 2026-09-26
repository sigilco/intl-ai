import 'package:build/build.dart';

import 'src/builder.dart' as src;

export 'src/builder.dart' show intlAiBuilder, IntlAiBuilder;
export 'src/options.dart' show IntlAiBuilderOptions;

/// Alias exported for consumers that prefer the `builder.dart` import path.
Builder intlAiBuilder(BuilderOptions options) => src.intlAiBuilder(options);
