import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'chatBot_model.dart';
//new-noor
import 'screen_savvy_prompt.dart';



class GeminiModel implements chatBot_model {
  final String apiKey;
  final String model;

  GeminiModel({required this.apiKey, this.model = 'gemini-2.5-flash-lite'});

 @override
String build_Prompt(String subject, String body, String from) {
  return buildScreenSavvyPrompt(
    subject: subject,
    from: from,
    body: body,
  );
}

  @override
  Future<Map<String, dynamic>> analyze(String subject, String body, String from) async {
    final prompt = build_Prompt(subject, body, from);
    return await _generateWithRetry(prompt);
  }

  // Retry mechanism
  Future<Map<String, dynamic>> _generateWithRetry(String prompt) async {
    const int maxRetries = 5;
    int attempt = 0;

    while (true) {
      try {
        return await _sendRequest(prompt);
      } catch (e) {
        attempt++;

        if (attempt >= maxRetries) {
          throw Exception("Gemini failed after $maxRetries retries: $e");
        }

        if (e.toString().contains("503") || e.toString().contains("UNAVAILABLE")) {
          final wait = Duration(seconds: attempt * 2);
          print("Gemini overloaded. Retrying in ${wait.inSeconds} seconds...");
          await Future.delayed(wait);
        } else {
          throw e;
        }
      }
    }
  }

  Future<Map<String, dynamic>> _sendRequest(String prompt) async {
    final url = Uri.parse(
        'https://generativelanguage.googleapis.com/v1beta/models/$model:generateContent?key=$apiKey');

    final response = await http
        .post(
          url,
          headers: {'Content-Type': 'application/json'},
          body: json.encode({
            "contents": [
              {
                "parts": [
                  {"text": prompt}
                ]
              }
            ]
          }),
        )
        .timeout(const Duration(seconds: 20));

    if (response.statusCode != 200) {
      throw Exception('Error ${response.statusCode}: ${response.body}');
    }

    final jsonResponse = jsonDecode(response.body);

    try {
      final text = jsonResponse["candidates"][0]["content"]["parts"][0]["text"];

      final jsonString = _extractJson(text);

      return jsonDecode(jsonString);
    } catch (e) {
      throw Exception("Failed to parse Gemini response: $jsonResponse");
    }
  }

  String _extractJson(String text) {
    final regex = RegExp(r'\{.*\}', dotAll: true);
    final match = regex.firstMatch(text);
    if (match != null) {
      return match.group(0)!;
    }
    throw Exception("No JSON found in text: $text");
  }
}

