import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'intl-ai Flutter Example',
      theme: ThemeData(colorSchemeSeed: Colors.blue),
      home: const HomeScreen(),
    );
  }
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  String _locale = Platform.localeName.split('_').first;
  Map<String, dynamic> _messages = {};

  @override
  void initState() {
    super.initState();
    _loadLocale(_locale);
  }

  Future<void> _loadLocale(String locale) async {
    final jsonString = await rootBundle.loadString('assets/locales/$locale.json');
    setState(() {
      _locale = locale;
      _messages = jsonDecode(jsonString) as Map<String, dynamic>;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('intl-ai Flutter Example'),
        actions: [
          DropdownButton<String>(
            value: _locale,
            items: const [
              DropdownMenuItem(value: 'en', child: Text('English')),
              DropdownMenuItem(value: 'es', child: Text('Español')),
            ],
            onChanged: (value) {
              if (value != null) _loadLocale(value);
            },
          ),
        ],
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(_messages['hello']?.toString() ?? '...', style: Theme.of(context).textTheme.headlineMedium),
            Text(_messages['goodbye']?.toString() ?? '...'),
          ],
        ),
      ),
    );
  }
}
