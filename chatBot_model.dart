abstract class chatBot_model {
  String build_Prompt(String subject, String body, String from);
  Future<Map<String, dynamic>> analyze(String subject, String body, String from);
}