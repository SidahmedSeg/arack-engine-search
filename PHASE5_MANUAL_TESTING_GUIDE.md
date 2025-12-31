# Phase 5 AI Features - Manual Testing Guide

## 🚀 Quick Start

This guide will help you manually test all Phase 5 AI features in the email application.

### Prerequisites

Before starting, ensure:
1. ✅ Backend email service is running: http://localhost:3001
   ```bash
   # Check service health
   curl http://localhost:3001/health
   # Expected: {"status":"ok"}
   ```

2. ✅ Frontend-email is running: http://localhost:5002
   ```bash
   cd frontend-email && npm run dev
   # Open browser to http://localhost:5002
   ```

3. ✅ You have a test email account provisioned
4. ✅ OpenAI API key is configured in `.env`

---

## 📋 Testing Checklist

Use this checklist to track your progress:

- [ ] 1. Smart Compose: Ghost text appears
- [ ] 2. Smart Compose: Tab key accepts suggestion
- [ ] 3. Smart Compose: Shift+Tab cycles backward
- [ ] 4. Smart Compose: Escape dismisses
- [ ] 5. Summarization: Button appears in email detail
- [ ] 6. Summarization: Summary generates with key points
- [ ] 7. Summarization: Toggle show/hide works
- [ ] 8. Priority Inbox: Page loads with ranked emails
- [ ] 9. Priority Inbox: Color-coded badges display
- [ ] 10. Priority Inbox: AI reasoning shows
- [ ] 11. Settings: Quota display shows all three features
- [ ] 12. Settings: Countdown timer updates
- [ ] 13. Settings: Progress bars are color-coded

---

## 🧪 Test 1: Smart Compose

### What It Does
AI-powered email completion that suggests how to continue your email as you type.

### How to Test

1. **Navigate to Inbox**
   - Open http://localhost:5002/inbox
   - You should see your mailbox list on the left

2. **Open Composer**
   - Click the blue "Compose" button
   - A compact composer window appears in the bottom-right

3. **Start Typing**
   - To: `test@example.com`
   - Subject: `Meeting Request`
   - Body: Type slowly: `"Hi, I wanted to schedule a meeting with you to discuss"`
   - **⏱️ Wait 2 seconds** (debounce delay)

4. **Observe Ghost Text**
   - You should see gray italic text appear
   - Hint text shows: "Press Tab to accept (1/3) or Esc to dismiss"
   - The suggestion completes your sentence professionally

5. **Test Keyboard Shortcuts**

   **Tab to Accept:**
   - Press `Tab` key
   - ✅ Text inserts into editor
   - ✅ Cursor moves to end
   - ✅ Next suggestion appears (2/3)

   **Shift+Tab to Go Back:**
   - Press `Shift+Tab`
   - ✅ Previous suggestion appears (1/3)

   **Escape to Dismiss:**
   - Press `Escape`
   - ✅ Ghost text disappears

6. **Test Context Awareness**
   - Compose a **reply** (click Reply on an email)
   - Type: `"Thanks for the update. I reviewed the"`
   - ✅ Suggestion reflects reply context
   - ✅ References subject/previous message

### Success Criteria
- ✅ Ghost text appears after 2 seconds of inactivity
- ✅ Tab, Shift+Tab, Escape work correctly
- ✅ Suggestions are professional and contextual
- ✅ No console errors

### Common Issues
- **No ghost text appears**: Check browser console for errors, verify backend is running
- **Text doesn't insert**: Make sure you're focused in the editor when pressing Tab
- **Too many API calls**: Debounce should prevent calls while typing continuously

---

## 🧪 Test 2: Email Summarization

### What It Does
AI generates a concise summary of email threads with key points and action items.

### How to Test

1. **Open an Email**
   - Navigate to /inbox
   - Click on any email in the list
   - Email detail pane opens on the right

2. **Find the Summarize Button**
   - Look at the header toolbar (top of detail pane)
   - You should see a button with Sparkles icon (✨) labeled "Summarize"
   - It's between the Star button and the More menu

3. **Click Summarize**
   - Click the "Summarize" button
   - ✅ Loading spinner appears
   - ✅ Button text changes to "Summarizing..."
   - ⏱️ Wait 2-5 seconds for API response

4. **View Summary Card**
   When complete, you should see:
   - ✅ Blue-themed card expands below email headers
   - ✅ **AI Summary** title with Sparkles icon
   - ✅ One-paragraph summary of the email
   - ✅ **Key Points:** section with bulleted items
   - ✅ **Action Items:** section with arrows (→)
   - ✅ Token count at bottom (e.g., "150 tokens used")
   - ✅ Button text changes to "Hide"

5. **Test Toggle**
   - Click "Hide" button
   - ✅ Summary card collapses
   - ✅ Button text changes back to "Summarize"
   - Click "Summarize" again
   - ✅ Summary reappears **instantly** (no new API call)
   - Open browser DevTools → Network tab to confirm no duplicate request

6. **Test with Email Thread**
   - Find an email that's part of a conversation (multiple messages)
   - Summarize it
   - ✅ Summary should include information from all emails in thread

### Success Criteria
- ✅ Summary is accurate and concise (1 paragraph)
- ✅ Key points are relevant to email content
- ✅ Action items correctly identified (if any)
- ✅ Caching works (no duplicate API calls)
- ✅ No errors in console

### Common Issues
- **Error message appears**: Check if backend OpenAI API key is configured
- **Summary doesn't make sense**: Email might be too short or have no meaningful content
- **Slow response**: OpenAI API latency varies; 2-10 seconds is normal

---

## 🧪 Test 3: Priority Inbox

### What It Does
AI ranks all your inbox emails by importance and urgency, showing highest priority first.

### How to Test

1. **Navigate to Priority Inbox**
   - From /inbox, look at the left sidebar
   - Below the "Compose" button, you should see "Priority Inbox" with Sparkles icon
   - Click it
   - ✅ URL changes to /priority
   - ✅ Page title shows "Priority Inbox" with Sparkles

2. **Wait for Ranking**
   - ⏱️ Initial load: Ranking happens automatically
   - You'll see "AI is ranking your emails..." with spinner
   - ⏱️ Wait 3-10 seconds (depends on number of emails)

3. **Observe Ranked List**
   Each email should have:
   - ✅ **Color-coded priority badge** (left side):
     - 🔴 **Red** badge with score 8-10: High priority
     - 🟠 **Orange** badge with score 6-7: Medium priority
     - ⚪ **Gray** badge with score 1-5: Low priority
   - ✅ **Numeric score** (1-10) displayed in badge
   - ✅ Sender, subject, preview text
   - ✅ **AI reasoning** below preview (blue text with Sparkles icon)
     - Example: "Urgent request from frequent contact"

4. **Verify Ranking Logic**
   Check if scores make sense:
   - ✅ Emails from your boss/frequent contacts → Higher scores
   - ✅ Emails with "urgent", "asap", "important" → Higher scores
   - ✅ Newsletters, promotional emails → Lower scores
   - ✅ Questions or action requests → Higher scores

5. **Test Refresh Rankings**
   - Click "Refresh Rankings" button (top right)
   - ✅ Button shows "Ranking..." during load
   - ✅ Button disabled while ranking
   - ✅ Scores may change slightly on refresh
   - ✅ No page reload

6. **Test Email Selection**
   - Click on any ranked email
   - ✅ Email detail opens (same as inbox)
   - Press `j` key
   - ✅ Moves to next email in priority order
   - Press `k` key
   - ✅ Moves to previous email

### Success Criteria
- ✅ Emails sorted by priority score (highest first)
- ✅ Color coding matches score ranges
- ✅ AI reasoning is descriptive and accurate
- ✅ Scores align with actual email importance
- ✅ Keyboard navigation works

### Common Issues
- **All scores are the same**: Backend might not have enough data about email frequency
- **Scores seem random**: AI model might need tuning; check AI reasoning for explanation
- **No emails appear**: Make sure you have emails in your inbox

---

## 🧪 Test 4: AI Quota Display (Settings)

### What It Does
Shows how many AI feature calls you've used today and when quotas reset.

### How to Test

1. **Navigate to Settings**
   - From any page, click the Settings icon (⚙️) in top-right header
   - ✅ URL changes to /settings
   - ✅ Page title shows "Settings"

2. **View AI Features Usage Section**
   You should see three cards:

   **Card 1: Smart Compose**
   - ✅ Title: "Smart Compose"
   - ✅ Description: "AI-powered email completion suggestions while you type"
   - ✅ Usage: "X / 50 calls" (e.g., "5 / 50 calls")
   - ✅ Progress bar (green if < 35, yellow if 35-45, red if ≥ 45)

   **Card 2: Email Summarization**
   - ✅ Title: "Email Summarization"
   - ✅ Description: "Generate concise summaries of email threads with key points"
   - ✅ Usage: "X / 20 calls"
   - ✅ Progress bar color matches usage percentage

   **Card 3: Priority Inbox**
   - ✅ Title: "Priority Inbox"
   - ✅ Description: "AI ranks your emails by importance and urgency"
   - ✅ Usage: "X / 10 calls"
   - ✅ Progress bar color matches usage percentage

3. **Test Countdown Timer**
   - Scroll to blue "Quota Reset" info card
   - ✅ Text: "All AI feature quotas reset daily at midnight UTC"
   - ✅ Countdown timer shows: "Xh Ym Zs" (e.g., "5h 23m 47s")
   - ⏱️ **Wait 10 seconds**
   - ✅ Timer counts down (seconds decrease)

4. **Test Usage Increments**
   - Go back to /inbox
   - Use Smart Compose (trigger a suggestion)
   - Return to /settings
   - ✅ Smart Compose counter increased by 1
   - ✅ Progress bar width increased

5. **Test Color Coding**
   To verify colors:
   - **Green bar**: < 70% usage (e.g., Smart Compose 34/50)
   - **Yellow bar**: 70-90% usage (e.g., Summarization 15/20)
   - **Red bar**: ≥ 90% usage (e.g., Priority 9/10)

6. **Check Account Info**
   - Scroll to "Account" section
   - ✅ Email address displayed correctly
   - ✅ Account ID shown in monospace font

### Success Criteria
- ✅ All three feature cards display
- ✅ Usage counters are accurate
- ✅ Progress bars visually match percentages
- ✅ Color coding is correct (green/yellow/red)
- ✅ Countdown timer updates every second
- ✅ Timer calculates correctly to midnight UTC

### Common Issues
- **Counters don't update**: Refresh the page to fetch latest data
- **Wrong time zone**: Timer should always count to midnight **UTC**, not local time
- **Progress bar too wide/narrow**: Check if percentage calculation matches usage/limit

---

## 🔄 Integration Test: Complete Workflow

Test all features together in one flow:

1. **Start at Priority Inbox** (/priority)
   - Note the highest priority email
   - ✅ Score, color, AI reasoning

2. **Open and Summarize**
   - Click the high-priority email
   - Click "Summarize"
   - ✅ Read the summary, key points, action items
   - ✅ Token count displayed

3. **Compose Reply with Smart Compose**
   - Click "Reply" button
   - Start typing a response
   - ⏱️ Wait for Smart Compose suggestion
   - Press `Tab` to accept
   - ✅ Suggestion inserted
   - Complete and send the email

4. **Check Quotas**
   - Navigate to /settings
   - ✅ Smart Compose: +1 (used in reply)
   - ✅ Summarization: +1 (used in detail)
   - ✅ Priority Ranking: +1 (page load)
   - ✅ Total: 3 API calls made

### Success Criteria
- ✅ All features work seamlessly together
- ✅ No errors or crashes
- ✅ Quota counters accurate
- ✅ User experience is smooth

---

## 🎨 Visual Test: Dark Mode

1. **Enable Dark Mode**
   - Go to /settings
   - Click the Moon icon button (top-right)
   - Page switches to dark theme

2. **Test AI Features in Dark Mode**
   - Navigate to /priority
   - ✅ Color-coded badges visible (red/orange/gray readable)
   - ✅ AI reasoning text (blue) is readable

3. **Test Composer**
   - Click Compose
   - Trigger Smart Compose
   - ✅ Ghost text is readable (gray on dark background)

4. **Test Summary**
   - Open an email
   - Summarize it
   - ✅ Blue summary card readable
   - ✅ Text contrasts properly

### Success Criteria
- ✅ All AI features render correctly in dark mode
- ✅ Blue accent colors remain visible
- ✅ Text is readable (no contrast issues)
- ✅ Icons are visible

---

## ⚠️ Error Scenarios to Test

### 1. Quota Exceeded
**How to trigger**:
- Make 50+ Smart Compose requests in one day
- Or 20+ Summarization requests
- Or 10+ Priority Ranking requests

**Expected behavior**:
- ✅ API returns HTTP 429 (Too Many Requests)
- ✅ User sees friendly error message
- ✅ Feature gracefully degrades (no crash)

### 2. Network Error
**How to trigger**:
- Stop the backend email service
- Try to summarize an email

**Expected behavior**:
- ✅ Error message appears
- ✅ "Try Again" button shown
- ✅ No white screen of death

### 3. Empty Inbox
**How to trigger**:
- Delete all emails
- Navigate to /priority

**Expected behavior**:
- ✅ "No emails to rank" message
- ✅ No API call made
- ✅ No errors

---

## 📊 Performance Checks

Open browser DevTools (F12) → Network tab during tests:

### Smart Compose
- ✅ No API calls while typing continuously
- ✅ API call exactly 2 seconds after last keystroke
- ✅ Only one call per typing session

### Summarization
- ✅ First summarize: API call visible
- ✅ Second show (after hide): **NO** new API call (cached)

### Priority Ranking
- ✅ Single batch API call for all emails
- ✅ Completes in < 5 seconds (for 50 emails)
- ✅ No UI freeze

---

## ✅ Final Verification

Before marking testing complete, verify:

1. **Backend Health**
   ```bash
   curl http://localhost:3001/health
   ```
   Should return: `{"status":"ok"}`

2. **Frontend Running**
   - http://localhost:5002 loads without errors

3. **All Features Tested**
   - [ ] Smart Compose: 6 sub-tests ✅
   - [ ] Summarization: 6 sub-tests ✅
   - [ ] Priority Inbox: 6 sub-tests ✅
   - [ ] Settings Quota: 6 sub-tests ✅
   - [ ] Integration: 1 test ✅
   - [ ] Dark Mode: 4 sub-tests ✅
   - [ ] Error Scenarios: 3 tests ✅
   - [ ] Performance: 3 checks ✅

4. **Browser Console**
   - No critical errors (404, 500, etc.)
   - Warnings about Svelte runes are OK (non-critical)

5. **Documentation Updated**
   - Mark tests as PASSED or FAILED in `PHASE5_AI_TESTING_PLAN.md`
   - Note any bugs found
   - Update `EMAIL_PHASE5_COMPLETE.md` with test results

---

## 🐛 Reporting Issues

If you find bugs during testing, document:

1. **Test Name**: Which test case failed
2. **Steps to Reproduce**: Exact steps taken
3. **Expected Result**: What should have happened
4. **Actual Result**: What actually happened
5. **Screenshots**: If applicable
6. **Browser Console Errors**: Copy any error messages
7. **Environment**: Browser version, OS, etc.

---

## 🎉 Success!

If all tests pass, Phase 5 AI features are ready for production!

Next steps:
- Phase 6: Advanced email features (contacts, labels, attachments, threading)
- Phase 7: Production hardening (security, DNS, performance, monitoring)

---

**Happy Testing! 🚀**
