import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'chatBot_model.dart';
import 'screen_savvy_prompt.dart';
import 'GeminiModel.dart';
import 'nextid.dart';


Future<void> main() async {
  final gemini = GeminiModel(
    apiKey: "AIzaSyDCxBTTLCJxeLDk7O-picwTZjO04C-PQq4"
  );

  try {
     final result = await gemini.analyze(
       "האפליקציה קורסת",
       "האפליקציה קורסת כשאני לוחצת על שמור",
       "customer@example.com",
     );

     print(" Result: $result");
    await createFromJson(result);


   } catch (e) {
     print(" Error: $e");
   }
 }
