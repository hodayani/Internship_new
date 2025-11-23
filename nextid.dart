import 'dart:io';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:googleapis_auth/auth_io.dart';

const serviceAccountPath = 'service_account.json';
const projectId = 'internshiptests-88a8a';

/// -----------------------------
/// ACCESS TOKEN
/// -----------------------------
Future<String> getAccessToken() async {
  final accountCredentials = ServiceAccountCredentials.fromJson(
      json.decode(await File(serviceAccountPath).readAsString()));
  final scopes = ['https://www.googleapis.com/auth/datastore'];
  final client = await clientViaServiceAccount(accountCredentials, scopes);
  final token = client.credentials.accessToken;
  client.close();
  return token.data;
}

/// -----------------------------
/// GET ALL EXISTING NUMBERS IN COLLECTION
/// -----------------------------
Future<List<int>> getAllExistingNumbers(String collection, String kind) async {
  final accessToken = await getAccessToken();
  final url =
      'https://firestore.googleapis.com/v1/projects/$projectId/databases/(default)/documents/$collection';

  final response = await http.get(
    Uri.parse(url),
    headers: {'Authorization': 'Bearer $accessToken'},
  );

  List<int> numbers = [];

  if (response.statusCode == 200) {
    final data = json.decode(response.body);
    if (data['documents'] != null) {
      for (var doc in data['documents']) {
        if (doc['fields'] != null && doc['fields']['id'] != null) {
          final idStr = doc['fields']['id']['stringValue'];
          if (idStr != null && idStr.startsWith(kind)) {
            final parts = idStr.split('-');
            if (parts.length == 3) {
              final numPart = int.tryParse(parts[2]);
              if (numPart != null) {
                numbers.add(numPart);
              }
            }
          }
        }
      }
    }
  }

  numbers.sort();
  return numbers;
}

/// -----------------------------
/// CREATE DOCUMENT FOR FEEDBACK OR BUG BASED ON JSON
/// -----------------------------
Future<void> createFromJson(Map<String, dynamic> jsonData, {String? status}) async {
  final kind = jsonData['category'] == 'bug' ? 'BUG' : 'FB';
  final collectionName = kind == 'BUG' ? 'bugs' : 'feedbacks';

  final existingNumbers = await getAllExistingNumbers(collectionName, kind);

  // מצא את המספר הקטן ביותר פנוי
  int nextNumber = 1;
  for (int i = 1;; i++) {
    if (!existingNumbers.contains(i)) {
      nextNumber = i;
      break;
    }
  }

  // צור את ה-ID בפורמט הרצוי
  final now = DateTime.now();
  final datePart =
      "${now.year}${now.month.toString().padLeft(2, '0')}${now.day.toString().padLeft(2, '0')}";
  final docId = "$kind-$datePart-${nextNumber.toString().padLeft(4, '0')}";

  // בנה את השדות לפי מה שביקשת
  Map<String, dynamic> fields = {
    'id': {'stringValue': docId},
    'kind': {'stringValue': jsonData['category'] ?? kind.toLowerCase()},
    'createdAt': {'timestampValue': DateTime.now().toUtc().toIso8601String()},
    'email.from': {'stringValue': jsonData['from'] ?? ''},
    'email.subject': {'stringValue': jsonData['subject'] ?? ''},
    'model.category': {'stringValue': jsonData['category'] ?? ''},
    'model.confidence': {'doubleValue': (jsonData['confidence'] ?? 0).toDouble()},
    'model.payload': {'stringValue': json.encode(jsonData)}, // שומר את כל ה-JSON
    'status': {'stringValue': status ?? 'open'},
    'links': {'mapValue': {}}, // ניתן למלא אחר כך
  };

  final accessToken = await getAccessToken();
  final url =
      'https://firestore.googleapis.com/v1/projects/$projectId/databases/(default)/documents/$collectionName/$docId';

  final response = await http.patch(
    Uri.parse(url),
    headers: {
      'Authorization': 'Bearer $accessToken',
      'Content-Type': 'application/json',
    },
    body: json.encode({'fields': fields}),
  );

  if (response.statusCode == 200) {
    if(kind=='BUG'){
      print('Thank you! We have received your report. Case number: $docId. The issue will be addressed as soon as possible.');
    }
    else{
      print('Thanks for the feedback! Case number: $docId. This helps us improve ScreenSavvy. If you have any more examples, we would be happy to hear them.');
    }
    print('✅ $docId נוצר בהצלחה');
  } else {
    throw Exception('Failed to create document: ${response.body}');
  }
}

/// -----------------------------
/// MAIN
/// -----------------------------
void main() async {
  // דוגמה JSON מה־Gemini
  final sampleJsonBug = {
    "category": "bug",
    "confidence": 0.9,
    "language": "en",
    "bug": {
      "type": "payments",
      "component": "credit_card",
      "severity": "critical",
      "steps_to_repro":
          "1. Attempt to make a credit card payment.\n2. Observe payment failure.",
      "os": null,
      "app_version": null
    },
    "feedback": {
      "topic": null,
      "summary": null,
      "nps_hint": null
    },
    "meta": {"pii_redacted": true, "detected_entities": ["email"]},
    "from": "customer@example.com",
    "subject": "Payment issue"
  };

  final sampleJsonFeedback = {
    "category": "feedback",
    "confidence": 0.8,
    "language": "en",
    "feedback": {
      "topic": "UI",
      "summary": "App is slow",
      "nps_hint": null
    },
    "meta": {"pii_redacted": true},
    "from": "user@example.com",
    "subject": "App performance"
  };

  try {
    await createFromJson(sampleJsonBug);
    await createFromJson(sampleJsonFeedback);
  } catch (e) {
    print('שגיאה: $e');
  }
}
