import 'GeminiModel.dart';
import 'test_emails.dart';

Future<void> runAllTests() async {
  final gemini = GeminiModel(
    apiKey: "", // ← put your key
  );

  for (var i = 0; i < testEmails.length; i++) {
    final t = testEmails[i];

    print("\n==============================");
    print("TEST ${i + 1}: ${t["subject"]}");
    print("==============================");

    try {
      final result = await gemini.analyze(
        t["subject"]!,
        t["body"]!,
        t["from"]!,
      );
      print(result);
    } catch (e) {
      print("Error in test ${i + 1}: $e");
    }

    // 🔥 IMPORTANT: delay to avoid 429 quota (15 req/min)
    await Future.delayed(const Duration(seconds: 5));
  }
}

Future<void> main() async {
  await runAllTests();
}
