final List<Map<String, String>> testEmails = [
  // 1. Payments - crash
  {
    "subject": "Payment keeps failing",
    "body": "Every time I try to pay for the premium plan, the app crashes after I click “Confirm Payment”.",
    "from": "user1@test.com"
  },

  // 2. Payments - rejected card
  {
    "subject": "My card isn’t accepted",
    "body": "It says 'payment error' but my card works in other apps.",
    "from": "sara93@example.com"
  },

  // 3. Login - no verification code
  {
    "subject": "I can't login",
    "body": "I enter my phone number but the code never arrives.",
    "from": "user2@test.com"
  },

  // 4. Login - incorrect code
  {
    "subject": "Verification code problem",
    "body": "The code arrives but the app says it's incorrect.",
    "from": "mariam@test.com"
  },

  // 5. Crash on startup
  {
    "subject": "App crashes immediately",
    "body": "I open the app and it closes after 1 second.",
    "from": "crashy@home.com"
  },

  // 6. Challenge crash
  {
    "subject": "Challenge crash",
    "body": "The app crashed when I tried to start the 'No Phone for 30 Minutes' challenge.",
    "from": "user@test.com"
  },

  // 7. Slow performance
  {
    "subject": "App is slow",
    "body": "Everything takes forever to load, even switching tabs.",
    "from": "slowmo@test.com"
  },

  // 8. Battery drain
  {
    "subject": "Battery drain",
    "body": "The app drains like 20% battery in one hour.",
    "from": "battery@test.com"
  },

  // 9. Notifications missing
  {
    "subject": "No notifications",
    "body": "I don’t get reminders for my challenges.",
    "from": "test@xyz.com"
  },

  // 10. Notifications delayed
  {
    "subject": "Notifications late",
    "body": "My notifications arrive 20 minutes late.",
    "from": "me@example.com"
  },

  // 11. UI layout broken
  {
    "subject": "Screen cuts off",
    "body": "Part of the settings screen is hidden under the bottom bar.",
    "from": "ui@test.com"
  },

  // 12. Text overlap
  {
    "subject": "Text overlap",
    "body": "On the home screen the challenge titles overlap each other.",
    "from": "visual@test.com"
  },

  // 13. Wrong screen time
  {
    "subject": "Incorrect screen time",
    "body": "The app says I used my phone for 5 hours, but I barely used it today.",
    "from": "track@test.com"
  },

  // 14. Tracking stopped
  {
    "subject": "Tracking stops randomly",
    "body": "After 2 pm today, the app stopped counting my usage completely.",
    "from": "monitor@test.com"
  },

  // 15. Photo verification rejected
  {
    "subject": "Photo verification error",
    "body": "I took a clear picture but the app said 'pose not recognized'.",
    "from": "camera@test.com"
  },

  // 16. Location verification wrong
  {
    "subject": "Location challenge issue",
    "body": "The app says I'm not at school even though I am.",
    "from": "geo@test.com"
  },

  // 17. Tablet issue
  {
    "subject": "Tablet issue",
    "body": "Most buttons don’t respond on my Samsung tablet.",
    "from": "tab@test.com"
  },

  // 18. Small screen issue
  {
    "subject": "Tiny buttons",
    "body": "On my old phone everything appears super tiny.",
    "from": "tiny@test.com"
  },

  // 19. Network error
  {
    "subject": "No internet?",
    "body": "I have WiFi but the app says 'network error'.",
    "from": "wifi@test.com"
  },

  // 20. Sync failure
  {
    "subject": "Sync problem",
    "body": "My progress doesn’t sync between my phone and tablet.",
    "from": "sync@test.com"
  },

  // 21. Feedback - dark mode request
  {
    "subject": "Dark mode please",
    "body": "Can you add dark mode? My eyes hurt at night.",
    "from": "user@test.com"
  },

  // 22. Feedback - weekly summary
  {
    "subject": "Weekly summary",
    "body": "I want a weekly usage report to see my progress.",
    "from": "summary@test.com"
  },

  // 23. Feedback - confusing onboarding
  {
    "subject": "Onboarding hard to understand",
    "body": "I was confused during the setup, too many screens.",
    "from": "feedback1@test.com"
  },

  // 24. Feedback - usability
  {
    "subject": "Hard to press",
    "body": "Some buttons are too small for my fingers.",
    "from": "usability@test.com"
  },

  // 25. Positive sentiment
  {
    "subject": "Amazing app!",
    "body": "I reduced my screen time by 40%, thank you!!",
    "from": "happy@test.com"
  },

  // 26. Negative sentiment
  {
    "subject": "Annoying",
    "body": "The app keeps bothering me with popups.",
    "from": "angry@test.com"
  },

  // 27. Feedback - more content
  {
    "subject": "Add more challenges",
    "body": "I finished all challenges, want new ones.",
    "from": "content@test.com"
  },

  // 28. Hebrew crash
  {
    "subject": "האפליקציה קורסת לי",
    "body": "כל פעם שאני פותחת את האפליקציה היא נסגרת אחרי שניה.",
    "from": "test@test.com"
  },

  // 29. Hebrew feedback
  {
    "subject": "אהבתי את האפליקציה",
    "body": "האתגרים ממש עוזרים לי להתרחק מהטלפון.",
    "from": "none@test.com"
  },

  // 30. Mixed language
  {
    "subject": "Notification problem",
    "body": "ההתראות לא מגיעות בזמן ואני מפספסת את ה-challenge.",
    "from": "hybrid@test.com"
  },

  // 31. PII test - phone + email
  {
    "subject": "Problem with login",
    "body": "My number is 0599912345 and my email is abcd@gmail.com, I can’t log in.",
    "from": "user@test.com"
  },

  // 32. ID number test
  {
    "subject": "Tracking issue",
    "body": "My ID is 207398123 and I think the tracking is broken.",
    "from": "me@test.com"
  },

  // 33. Ambiguous
  {
    "subject": "Something wrong",
    "body": "The app acts weird sometimes but I'm not sure why.",
    "from": "unclear@test.com"
  },

  // 34. Empty body
  {
    "subject": "Help",
    "body": "",
    "from": "empty@test.com"
  },

  // 35. Bug disguised as feedback
  {
    "subject": "I think something is off",
    "body": "The progress bar doesn’t move even when I complete tasks.",
    "from": "confuse@test.com"
  },

  // 36. Feedback disguised as bug
  {
    "subject": "Great app but small issue",
    "body": "I love it but the login sometimes doesn’t work.",
    "from": "praise@test.com"
  },

  // 37. UI feedback mistaken as bug
  {
    "subject": "Problem?",
    "body": "No problem but I think the UI could be prettier.",
    "from": "ui@test.com"
  },

  // 38. Long body test
  {
    "subject": "Long message",
    "body": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. " * 10,
    "from": "long@test.com"
  },

  // 39. Null inference bug
  {
    "subject": "Something broke",
    "body": "It just stopped working.",
    "from": "bug@test.com"
  },

  // 40. Very vague (force low confidence)
  {
    "subject": "glitch?",
    "body": "idk something seems weird",
    "from": "vague@test.com"
  },

  // 41. Localization bug
  {
    "subject": "Hebrew text alignment issue",
    "body": "In Hebrew mode, some text is still left-aligned and overlaps with icons.",
    "from": "locale@test.com"
  },

  // 42. Accessibility - contrast
  {
    "subject": "Hard to read text",
    "body": "The grey text on white background is too low contrast for me.",
    "from": "access@test.com"
  },

  // 43. Accessibility - screen reader
  {
    "subject": "Screen reader issues",
    "body": "My screen reader doesn’t read the challenge titles correctly.",
    "from": "reader@test.com"
  },

  // 44. Pricing feedback
  {
    "subject": "Premium too expensive",
    "body": "I like the app but I think the premium subscription price is too high.",
    "from": "price@test.com"
  },

  // 45. Neutral feedback
  {
    "subject": "Just sharing my experience",
    "body": "The app is okay, not amazing, not terrible.",
    "from": "neutral@test.com"
  },

  // 46. Content feedback - low relevance
  {
    "subject": "Challenges don’t fit me",
    "body": "Most challenges feel more like they are for kids, not for university students.",
    "from": "fit@test.com"
  },

  // 47. Onboarding bug + feedback mix
  {
    "subject": "Setup stuck",
    "body": "During onboarding the spinner never ends, and also the explanations are a bit unclear.",
    "from": "mix@test.com"
  },

  // 48. Multiple issues in one
  {
    "subject": "Many issues",
    "body": "Notifications are late, tracking is off, and I also wish there was a focus music feature.",
    "from": "multi@test.com"
  },

  // 49. Strong negative sentiment bug
  {
    "subject": "I hate this app now",
    "body": "It logged me out during an important challenge and I lost all my progress.",
    "from": "rage@test.com"
  },

  // 50. Strong positive sentiment + feature
  {
    "subject": "Best app ever but one request",
    "body": "I love it so much, I just wish there was a group challenge mode for my class.",
    "from": "fan@test.com"
  },
 // ============================================
  // OS DETECTION EDGE CASES (51-60)
  // ============================================
  
  // 51. Brand name only (Samsung)
  {
    "subject": "App won't open on Samsung",
    "body": "I have a Samsung and the app crashes on startup.",
    "from": "samsung@test.com"
  },

  // 52. Brand name only (iPhone)
  {
    "subject": "iPhone issue",
    "body": "My iPhone keeps showing errors when I try to log in.",
    "from": "iphone@test.com"
  },

  // 53. Explicit Android version
  {
    "subject": "Bug on Android 13",
    "body": "Running Android 13 and notifications don't work.",
    "from": "android13@test.com"
  },

  // 54. Explicit iOS version
  {
    "subject": "iOS 17 crash",
    "body": "On iOS 17.2 the app crashes when opening challenges.",
    "from": "ios17@test.com"
  },

  // 55. Device type only (tablet)
  {
    "subject": "Tablet display issue",
    "body": "On my tablet the buttons are cut off.",
    "from": "tablet@test.com"
  },

  // 56. Web browser mention
  {
    "subject": "Web version problem",
    "body": "Using the web version on Chrome and tracking doesn't update.",
    "from": "web@test.com"
  },

  // 57. Multiple devices, no OS
  {
    "subject": "Works on one device, not the other",
    "body": "It works on my phone but not on my tablet.",
    "from": "multidevice@test.com"
  },

  // 58. Generic "mobile" mention
  {
    "subject": "Mobile app issue",
    "body": "The mobile app keeps logging me out randomly.",
    "from": "mobile@test.com"
  },

  // 59. Explicit iPad (should be iOS)
  {
    "subject": "iPad Pro issue",
    "body": "My iPad Pro on iOS 16 has a screen freeze problem.",
    "from": "ipad@test.com"
  },

  // 60. Android phone (should extract Android)
  {
    "subject": "Android phone bug",
    "body": "I'm using an Android phone and challenges won't start.",
    "from": "androidphone@test.com"
  },

  // ============================================
  // SEVERITY CLASSIFICATION EDGE CASES (61-70)
  // ============================================

  // 61. Onboarding completely blocked (should be critical)
  {
    "subject": "Can't complete signup",
    "body": "The signup process freezes at step 3 and I can't create an account at all.",
    "from": "blocked@test.com"
  },

  // 62. Intermittent crash (should be high)
  {
    "subject": "App crashes sometimes",
    "body": "About 3 times a day the app suddenly closes while I'm using it.",
    "from": "intermittent@test.com"
  },

  // 63. Feature works but slow (should be medium)
  {
    "subject": "Challenges load slowly",
    "body": "Challenges eventually load but it takes 10-15 seconds every time.",
    "from": "slow@test.com"
  },

  // 64. Minor visual glitch (should be low)
  {
    "subject": "Small animation glitch",
    "body": "When I complete a challenge, the animation stutters a bit.",
    "from": "glitch@test.com"
  },

  // 65. Payment declined but not app crash (should be high)
  {
    "subject": "Payment keeps declining",
    "body": "Every payment attempt is declined even though my card works everywhere else.",
    "from": "payment@test.com"
  },

  // 66. Data loss (should be critical)
  {
    "subject": "Lost all my progress",
    "body": "After the update all my challenge history and progress disappeared.",
    "from": "dataloss@test.com"
  },

  // 67. Login works after retry (should be medium)
  {
    "subject": "Login fails first try",
    "body": "I have to try logging in 2-3 times before it works.",
    "from": "retry@test.com"
  },

  // 68. Core feature broken (should be high)
  {
    "subject": "Screen time tracking stopped",
    "body": "The app hasn't tracked my screen time for 3 days now.",
    "from": "corefeature@test.com"
  },

  // 69. Notification typo (should be low)
  {
    "subject": "Typo in notification",
    "body": "The notification says 'chalenge' instead of 'challenge'.",
    "from": "typo@test.com"
  },

  // 70. Verification system completely broken (should be critical)
  {
    "subject": "Can't verify any challenges",
    "body": "No matter what photo I take, it says verification failed. I can't complete any challenges.",
    "from": "verification@test.com"
  },

  // ============================================
  // BUG vs FEEDBACK BORDERLINE CASES (71-80)
  // ============================================

  // 71. UI broken (should be bug)
  {
    "subject": "Buttons don't work",
    "body": "The save button in settings doesn't respond when I tap it.",
    "from": "broken@test.com"
  },

  // 72. UI aesthetics only (should be feedback)
  {
    "subject": "Color scheme",
    "body": "The app works fine but I think the colors could be more vibrant.",
    "from": "colors@test.com"
  },

  // 73. Hard to understand UI (should be feedback usability)
  {
    "subject": "Confusing layout",
    "body": "I can use the app but the menu structure is confusing.",
    "from": "confusing@test.com"
  },

  // 74. Button too small causing errors (should be bug)
  {
    "subject": "Keep pressing wrong button",
    "body": "The buttons are so small that I always press the wrong one and it triggers errors.",
    "from": "smallbutton@test.com"
  },

  // 75. Feature request framed as problem (should be feedback)
  {
    "subject": "Missing export feature",
    "body": "I wish I could export my data but there's no button for it.",
    "from": "export@test.com"
  },

  // 76. Performance complaint with workaround (should be bug medium)
  {
    "subject": "Slow unless I restart",
    "body": "After using the app for an hour it gets really slow, but restarting it fixes it.",
    "from": "workaround@test.com"
  },

  // 77. Accessibility barrier (should be bug)
  {
    "subject": "Can't use with VoiceOver",
    "body": "VoiceOver doesn't read any of the buttons so I can't navigate the app.",
    "from": "voiceover@test.com"
  },

  // 78. Font size preference (should be feedback)
  {
    "subject": "Font too small for me",
    "body": "The font works but I personally prefer larger text.",
    "from": "fontsize@test.com"
  },

  // 79. Feature exists but hard to find (should be feedback usability)
  {
    "subject": "Can't find settings",
    "body": "I know settings exist but I can never find where they are.",
    "from": "hidden@test.com"
  },

  // 80. Design makes feature unusable (should be bug ui/ux)
  {
    "subject": "Can't read white text",
    "body": "The white text on light background is impossible to read.",
    "from": "unreadable@test.com"
  },

  // ============================================
  // MULTI-ISSUE EMAILS (81-90)
  // ============================================

  // 81. Critical bug + praise
  {
    "subject": "Love the app but major issue",
    "body": "This app is amazing but I just lost all my data after the last update.",
    "from": "lostdata@test.com"
  },

  // 82. Bug + feature request
  {
    "subject": "Tracking bug and idea",
    "body": "The tracking stops at midnight every day, also it would be cool to have weekly goals.",
    "from": "bugidea@test.com"
  },

  // 83. Multiple bugs (different severity)
  {
    "subject": "Several problems",
    "body": "The app is slow, notifications are delayed, and sometimes it crashes when opening.",
    "from": "multiple@test.com"
  },

  // 84. Feedback + minor bug mention
  {
    "subject": "Feature suggestion",
    "body": "I'd love to see dark mode added. Also there's a tiny spelling error in the help section.",
    "from": "suggestion@test.com"
  },

  // 85. Positive feedback + usability issue
  {
    "subject": "Great progress, one complaint",
    "body": "I've reduced screen time by 50%! But the statistics page is hard to understand.",
    "from": "progress@test.com"
  },

  // 86. Two separate bugs
  {
    "subject": "Login and payment issues",
    "body": "I can't log in with Google, and when I try to subscribe it says payment error.",
    "from": "twoproblems@test.com"
  },

  // 87. Bug with usage context
  {
    "subject": "Problem during commute",
    "body": "When I'm on the bus the app doesn't track properly, but I also think offline mode would be helpful.",
    "from": "commute@test.com"
  },

  // 88. Critical bug buried in praise
  {
    "subject": "Mostly great!",
    "body": "I love everything about this app, the design is beautiful, the challenges are fun, oh and by the way I can't make payments at all.",
    "from": "buried@test.com"
  },

  // 89. Feedback with bug implications
  {
    "subject": "Need better error messages",
    "body": "When something goes wrong, the error message just says 'Error' with no details.",
    "from": "errors@test.com"
  },

  // 90. Multiple feedbacks
  {
    "subject": "Three suggestions",
    "body": "Add dark mode, weekly summaries, and social sharing features please!",
    "from": "threeideas@test.com"
  },

  // ============================================
  // PII VARIATIONS (91-100)
  // ============================================

  // 91. Multiple emails
  {
    "subject": "Account issue",
    "body": "I have two accounts, one@gmail.com and two@outlook.com, both are broken.",
    "from": "multi@test.com"
  },

  // 92. Phone number variations
  {
    "subject": "Can't receive SMS",
    "body": "My numbers are 052-1234567 and +972-50-1234567 but code never arrives.",
    "from": "phones@test.com"
  },

  // 93. ID number in Hebrew format
  {
    "subject": "בעיית זיהוי",
    "body": "מספר הזהות שלי הוא 123456789 ואני לא יכול להתחבר.",
    "from": "id@test.com"
  },

  // 94. Email in body and from
  {
    "subject": "Login help",
    "body": "I'm trying to log in with myemail@test.com but it's not working.",
    "from": "myemail@test.com"
  },

  // 95. Credit card mention (not a bug, but PII adjacent)
  {
    "subject": "Payment issue",
    "body": "My credit card ending in 1234 keeps getting declined.",
    "from": "card@test.com"
  },

  // 96. Name + phone
  {
    "subject": "Account recovery",
    "body": "My name is John Smith and my phone is 054-9876543, can you help?",
    "from": "john@test.com"
  },

  // 97. Address mention
  {
    "subject": "Delivery issue",
    "body": "I'm at 123 Main Street, Jerusalem and the location verification isn't working.",
    "from": "address@test.com"
  },

  // 98. No PII at all
  {
    "subject": "General question",
    "body": "How do I reset my progress?",
    "from": "nopii@test.com"
  },

  // 99. International phone
  {
    "subject": "International user",
    "body": "I'm calling from +44-20-1234-5678 and can't get the verification SMS.",
    "from": "intl@test.com"
  },

  // 100. Mixed PII types
  {
    "subject": "Complete profile",
    "body": "My details are: ID 987654321, phone 050-1111111, email user@mail.com",
    "from": "complete@test.com"
  },

  // ============================================
  // HEBREW LANGUAGE TESTS (101-110)
  // ============================================

  // 101. Pure Hebrew bug report
  {
    "subject": "תקלה חמורה",
    "body": "האפליקציה מתקעקעת כל פעם שאני מנסה לשלם. זה קורה כבר שבוע.",
    "from": "hebrew1@test.com"
  },

  // 102. Hebrew feedback positive
  {
    "subject": "תודה רבה",
    "body": "האפליקציה פשוט מעולה! הצלחתי להפחית את זמן המסך ב-60%.",
    "from": "hebrew2@test.com"
  },

  // 103. Hebrew feature request
  {
    "subject": "בקשת תכונה",
    "body": "אשמח אם תוסיפו מצב חשוך כי זה כואב לי לעיניים בלילה.",
    "from": "hebrew3@test.com"
  },

  // 104. Hebrew + English technical terms
  {
    "subject": "בעיה עם ה-tracking",
    "body": "ה-screen time tracking לא עובד לי מאתמול. ה-app לא מעדכן את הנתונים.",
    "from": "hebrew4@test.com"
  },

  // 105. Hebrew with version info
  {
    "subject": "קריסה בגרסה 2.5",
    "body": "אני משתמש בגרסה 2.5.1 על אנדרואיד 14 והאפליקציה קורסת כשאני פותח אתגרים.",
    "from": "hebrew5@test.com"
  },

  // 106. Hebrew complaint (detractor)
  {
    "subject": "מאוכזב",
    "body": "האפליקציה הזאת פשוט לא עובדת. התקלות כל הזמן וזה מעצבן מאוד.",
    "from": "hebrew6@test.com"
  },

  // 107. Hebrew neutral feedback
  {
    "subject": "משוב כללי",
    "body": "האפליקציה בסדר, לא משהו מיוחד. יש מקום לשיפור.",
    "from": "hebrew7@test.com"
  },

  // 108. Hebrew accessibility issue
  {
    "subject": "בעיית נגישות",
    "body": "הטקסט קטן מדי ואני לא מצליח לקרוא אותו בלי משקפיים.",
    "from": "hebrew8@test.com"
  },

  // 109. Hebrew performance complaint
  {
    "subject": "ביצועים איטיים",
    "body": "האפליקציה איטית מאוד. לוקח דקות עד שהיא נטענת.",
    "from": "hebrew9@test.com"
  },

  // 110. Hebrew multi-issue
  {
    "subject": "כמה בעיות",
    "body": "ההודעות מאחרות, המעקב לא מדויק, וגם הייתי רוצה תכונת ייצוא נתונים.",
    "from": "hebrew10@test.com"
  },

  // ============================================
  // CONFIDENCE TESTS - AMBIGUOUS CASES (111-120)
  // ============================================

  // 111. Extremely vague
  {
    "subject": "not working",
    "body": "it doesn't work",
    "from": "vague1@test.com"
  },

  // 112. Single word
  {
    "subject": "help",
    "body": "broken",
    "from": "vague2@test.com"
  },

  // 113. Question without context
  {
    "subject": "Why?",
    "body": "Why does this keep happening?",
    "from": "vague3@test.com"
  },

  // 114. Emotional but no details
  {
    "subject": "So frustrated!!!",
    "body": "This is the worst experience ever. I can't believe it.",
    "from": "vague4@test.com"
  },

  // 115. Cryptic message
  {
    "subject": "Issue",
    "body": "The thing with the thing isn't doing the thing.",
    "from": "vague5@test.com"
  },

  // 116. Just a complaint, no specifics
  {
    "subject": "Not happy",
    "body": "I'm really not satisfied with this.",
    "from": "vague6@test.com"
  },

  // 117. Spam-like
  {
    "subject": "URGENT!!!",
    "body": "NEED HELP NOW!!!",
    "from": "vague7@test.com"
  },

  // 118. Unclear pronoun reference
  {
    "subject": "It's broken",
    "body": "That part where you do the thing is broken.",
    "from": "vague8@test.com"
  },

  // 119. Reply without original context
  {
    "subject": "Re: Previous issue",
    "body": "Yes that's still happening.",
    "from": "vague9@test.com"
  },

  // 120. Sarcasm without clarity
  {
    "subject": "Great job...",
    "body": "Real nice update you guys released.",
    "from": "vague10@test.com"
  },

  // ============================================
  // COMPONENT-SPECIFIC TESTS (121-130)
  // ============================================

  // 121. Onboarding specific
  {
    "subject": "Stuck on welcome screen",
    "body": "I'm on the first screen where it asks for permissions and the Next button doesn't work.",
    "from": "onboard1@test.com"
  },

  // 122. Profile specific
  {
    "subject": "Can't update profile",
    "body": "When I try to change my avatar, it doesn't save.",
    "from": "profile1@test.com"
  },

  // 123. Search specific
  {
    "subject": "Search not working",
    "body": "I search for challenges but nothing shows up even though they exist.",
    "from": "search1@test.com"
  },

  // 124. Video player specific
  {
    "subject": "Videos won't play",
    "body": "Tutorial videos show a black screen and don't play.",
    "from": "video1@test.com"
  },

  // 125. Push notifications specific
  {
    "subject": "Push notifications broken",
    "body": "I enabled notifications in settings but never receive any push alerts.",
    "from": "push1@test.com"
  },

  // 126. Analytics/stats specific
  {
    "subject": "Statistics wrong",
    "body": "My weekly report shows 0 hours but I used the app every day.",
    "from": "analytics1@test.com"
  },

  // 127. Backend API specific
  {
    "subject": "Server error message",
    "body": "I keep getting 'Server error 500' when trying to sync.",
    "from": "backend1@test.com"
  },

  // 128. Sync specific
  {
    "subject": "Cross-device sync failed",
    "body": "I completed challenges on my phone but they don't show on my tablet.",
    "from": "sync1@test.com"
  },

  // 129. Settings specific
  {
    "subject": "Settings reset automatically",
    "body": "Every time I close the app, my settings go back to default.",
    "from": "settings1@test.com"
  },

  // 130. Checkout/payment flow specific
  {
    "subject": "Checkout freeze",
    "body": "At the payment summary page, the app freezes when I click Confirm.",
    "from": "checkout1@test.com"
  },

  // ============================================
  // EDGE CASES & SPECIAL SCENARIOS (131-150)
  // ============================================

  // 131. App version mentioned
  {
    "subject": "Bug in version 3.2.1",
    "body": "After updating to 3.2.1, notifications stopped working.",
    "from": "version@test.com"
  },

  // 132. Works on WiFi, not on cellular
  {
    "subject": "Only works on WiFi",
    "body": "The app functions fine on WiFi but fails on cellular data.",
    "from": "network@test.com"
  },

  // 133. Time-specific bug
  {
    "subject": "Midnight bug",
    "body": "Every day at exactly midnight, the app crashes.",
    "from": "midnight@test.com"
  },

  // 134. After-update regression
  {
    "subject": "Worked before update",
    "body": "Everything was fine until yesterday's update, now tracking is broken.",
    "from": "regression@test.com"
  },

  // 135. Competitor comparison
  {
    "subject": "Not as good as competitor",
    "body": "App X has better challenge variety, you should add more.",
    "from": "competitor@test.com"
  },

  // 136. School/institutional use
  {
    "subject": "School deployment issue",
    "body": "We're trying to use this for our entire school but bulk account creation fails.",
    "from": "school@test.com"
  },

  // 137. Privacy concern
  {
    "subject": "Privacy question",
    "body": "Why does the app need access to my photos? This seems excessive.",
    "from": "privacy@test.com"
  },

  // 138. Battery drain specific
  {
    "subject": "Battery drains overnight",
    "body": "Even when I'm not using the app, it drains 30% battery overnight.",
    "from": "batterydrain@test.com"
  },

  // 139. Storage space issue
  {
    "subject": "Takes too much space",
    "body": "The app is using 2GB of storage on my phone, is this normal?",
    "from": "storage@test.com"
  },

  // 140. Geolocation accuracy
  {
    "subject": "Location is always wrong",
    "body": "The app thinks I'm 5km away from where I actually am.",
    "from": "geo@test.com"
  },

  // 141. Integration with other apps
  {
    "subject": "Spotify integration broken",
    "body": "The app won't connect to my Spotify account for music challenges.",
    "from": "integration@test.com"
  },

  // 142. Calendar integration
  {
    "subject": "Calendar sync issue",
    "body": "My Google Calendar events don't show up in the app.",
    "from": "calendar@test.com"
  },

  // 143. Screenshot functionality
  {
    "subject": "Can't take screenshots",
    "body": "When I try to screenshot my progress, the screen goes black.",
    "from": "screenshot@test.com"
  },

  // 144. Share functionality
  {
    "subject": "Sharing doesn't work",
    "body": "I try to share my achievements but nothing happens.",
    "from": "share@test.com"
  },

  // 145. Widget issue
  {
    "subject": "Widget not updating",
    "body": "The home screen widget shows old data from yesterday.",
    "from": "widget@test.com"
  },

  // 146. Landscape mode
  {
    "subject": "Broken in landscape",
    "body": "When I rotate my phone to landscape, everything overlaps.",
    "from": "landscape@test.com"
  },

  // 147. Tablet optimization
  {
    "subject": "Not optimized for tablets",
    "body": "On my tablet everything looks stretched and ugly.",
    "from": "tabletopt@test.com"
  },

  // 148. Parent/child account
  {
    "subject": "Parental controls missing",
    "body": "I want to monitor my child's usage but there's no parent dashboard.",
    "from": "parent@test.com"
  },

  // 149. Subscription management
  {
    "subject": "Can't cancel subscription",
    "body": "I'm trying to cancel my premium subscription but can't find the option.",
    "from": "subscription@test.com"
  },

  // 150. Trial period confusion
  {
    "subject": "Free trial charged me",
    "body": "I thought there was a 7-day free trial but I got charged immediately.",
    "from": "trial@test.com"
  },
];
