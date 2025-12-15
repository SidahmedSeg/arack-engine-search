# Phase 5 AI Features - End-to-End Testing Plan

## Testing Environment

**Frontend**: http://localhost:5002
**Backend Email Service**: http://localhost:3001
**Test Date**: December 16, 2025

### Prerequisites
- ✅ Backend email service running on port 3001
- ✅ Frontend-email dev server running on port 5002
- ✅ OpenAI API key configured
- ✅ Test email account provisioned
- ✅ Card component imports fixed

---

## Test Cases

### 1. Smart Compose Feature

#### Test 1.1: Ghost Text Display
**Steps**:
1. Navigate to http://localhost:5002/inbox
2. Click "Compose" button
3. Fill in:
   - To: test@example.com
   - Subject: Meeting Request
4. Start typing in the body: "Hi, I wanted to schedule a meeting with you to discuss"
5. Wait 2 seconds (debounce delay)

**Expected Results**:
- ✅ Loading indicator appears briefly
- ✅ Ghost text overlay appears in gray italic
- ✅ Hint text shows: "Press Tab to accept (1/3) or Esc to dismiss"
- ✅ Suggestion completes the sentence naturally and professionally

**Actual Results**:
- [ ] Test not yet run

**Notes**:

---

#### Test 1.2: Keyboard Shortcuts
**Steps**:
1. Continue from Test 1.1 with ghost text visible
2. Press `Tab` key

**Expected Results**:
- ✅ Ghost text inserts into editor
- ✅ Cursor moves to end of inserted text
- ✅ Next suggestion appears (2/3)

**Actual Results**:
- [ ] Test not yet run

**Steps (continued)**:
3. Press `Shift+Tab`

**Expected Results**:
- ✅ Previous suggestion appears (1/3)

**Actual Results**:
- [ ] Test not yet run

**Steps (continued)**:
4. Press `Escape`

**Expected Results**:
- ✅ Ghost text disappears
- ✅ Hint text disappears
- ✅ Suggestions dismissed

**Actual Results**:
- [ ] Test not yet run

---

#### Test 1.3: Context-Aware Suggestions
**Steps**:
1. Compose new email
2. To: john@company.com
3. Subject: RE: Project Update
4. Type: "Thanks for the update. I reviewed the"

**Expected Results**:
- ✅ Suggestions reflect reply context (is_reply: true)
- ✅ Suggestions reference subject context
- ✅ Professional tone maintained

**Actual Results**:
- [ ] Test not yet run

---

#### Test 1.4: Quota Enforcement
**Steps**:
1. Make 50 Smart Compose requests (trigger suggestions 50 times)
2. On 51st request, check response

**Expected Results**:
- ✅ After 50 calls, returns HTTP 429 (Too Many Requests)
- ✅ Error message displayed to user
- ✅ Feature gracefully degrades (no ghost text)

**Actual Results**:
- [ ] Test not yet run

---

### 2. Email Summarization Feature

#### Test 2.1: Summarize Button Display
**Steps**:
1. Navigate to /inbox
2. Click on any email message
3. Observe email detail header

**Expected Results**:
- ✅ "Summarize" button visible in header toolbar
- ✅ Sparkles icon displayed
- ✅ Button positioned near other action buttons (Archive, Delete, Star)

**Actual Results**:
- [ ] Test not yet run

---

#### Test 2.2: Generate Summary
**Steps**:
1. Continue from Test 2.1
2. Click "Summarize" button
3. Wait for API response

**Expected Results**:
- ✅ Loading spinner appears during API call
- ✅ Blue-themed card expands below email headers
- ✅ Summary displayed in one paragraph
- ✅ "Key Points:" section with bulleted list
- ✅ "Action Items:" section with arrow-prefixed items
- ✅ Token count shown at bottom (e.g., "150 tokens used")
- ✅ Button text changes to "Hide"

**Actual Results**:
- [ ] Test not yet run

---

#### Test 2.3: Toggle Summary Visibility
**Steps**:
1. Continue from Test 2.2 with summary visible
2. Click "Hide" button
3. Click "Summarize" button again

**Expected Results**:
- ✅ First click: Summary card collapses
- ✅ Button text changes to "Summarize"
- ✅ Second click: Summary card re-appears instantly (cached)
- ✅ No API call made (check network tab)

**Actual Results**:
- [ ] Test not yet run

---

#### Test 2.4: Multi-Email Thread Summarization
**Steps**:
1. Find email thread with 3+ messages
2. Open thread
3. Click "Summarize"

**Expected Results**:
- ✅ Summary includes information from all emails in thread
- ✅ Chronological organization
- ✅ Sender attribution
- ✅ Key decisions and action items identified

**Actual Results**:
- [ ] Test not yet run

---

#### Test 2.5: Error Handling
**Steps**:
1. Disconnect network or stop backend
2. Click "Summarize" on an email
3. Wait for timeout

**Expected Results**:
- ✅ Error message displayed in red
- ✅ "Try Again" button appears
- ✅ No crash or white screen
- ✅ Clicking "Try Again" retries API call

**Actual Results**:
- [ ] Test not yet run

---

### 3. Priority Inbox Feature

#### Test 3.1: Priority Inbox Route Access
**Steps**:
1. Navigate to /inbox
2. Click "Priority Inbox" button in left sidebar
3. Observe URL and page content

**Expected Results**:
- ✅ URL changes to /priority
- ✅ Page title shows "Priority Inbox" with Sparkles icon
- ✅ "AI-ranked emails by importance and urgency" subtitle visible
- ✅ "Refresh Rankings" button in header

**Actual Results**:
- [ ] Test not yet run

---

#### Test 3.2: Email Ranking Display
**Steps**:
1. Continue from Test 3.1
2. Wait for ranking to complete
3. Observe email list

**Expected Results**:
- ✅ Emails sorted by priority score (highest first)
- ✅ Each email has color-coded badge:
  - Red badge (score ≥ 8): High priority
  - Orange badge (score ≥ 6): Medium priority
  - Gray badge (score < 6): Low priority
- ✅ Numeric score (1-10) displayed in badge
- ✅ AI reasoning shown below each email (e.g., "Urgent request from frequent contact")
- ✅ Blue sparkles icon next to reasoning

**Actual Results**:
- [ ] Test not yet run

---

#### Test 3.3: Ranking Accuracy
**Steps**:
1. Identify emails that should be high priority:
   - From frequent sender
   - Contains urgency keywords ("urgent", "asap", "important")
   - Has questions or action requests
2. Check their priority scores

**Expected Results**:
- ✅ High priority emails score ≥ 7
- ✅ Newsletter/promotional emails score ≤ 5
- ✅ AI reasoning accurately reflects email content
- ✅ Scores make intuitive sense

**Actual Results**:
- [ ] Test not yet run

---

#### Test 3.4: Refresh Rankings
**Steps**:
1. On /priority page
2. Click "Refresh Rankings" button
3. Wait for completion

**Expected Results**:
- ✅ "Ranking..." loading text appears
- ✅ Button disabled during loading
- ✅ Scores may change based on new analysis
- ✅ Page updates with new rankings
- ✅ No page reload required

**Actual Results**:
- [ ] Test not yet run

---

#### Test 3.5: Email Selection and Navigation
**Steps**:
1. On /priority page
2. Click on a high-priority email
3. Use `j` keyboard shortcut

**Expected Results**:
- ✅ Email detail opens (same as inbox)
- ✅ `j` moves to next ranked email
- ✅ `k` moves to previous ranked email
- ✅ Order maintained by priority score

**Actual Results**:
- [ ] Test not yet run

---

### 4. AI Quota Display in Settings

#### Test 4.1: Settings Page Navigation
**Steps**:
1. From /inbox or /priority
2. Click Settings icon in top-right header
3. Observe page load

**Expected Results**:
- ✅ URL changes to /settings
- ✅ Page title shows "Settings" with Settings icon
- ✅ Back button appears in header
- ✅ "AI Features Usage" section visible

**Actual Results**:
- [ ] Test not yet run

---

#### Test 4.2: Quota Display
**Steps**:
1. Continue from Test 4.1
2. Observe AI Features Usage section

**Expected Results**:
- ✅ Three cards displayed:
  1. Smart Compose
  2. Email Summarization
  3. Priority Inbox
- ✅ Each card shows:
  - Feature name and description
  - "X/Y calls today" (e.g., "23/50 calls")
  - Progress bar with width matching usage percentage
  - Color-coded: Green (<70%), Yellow (70-90%), Red (≥90%)

**Actual Results**:
- [ ] Test not yet run

---

#### Test 4.3: Live Countdown Timer
**Steps**:
1. Scroll to "Quota Reset" info card
2. Observe countdown timer for 10 seconds

**Expected Results**:
- ✅ Blue info card with Sparkles icon
- ✅ Text: "All AI feature quotas reset daily at midnight UTC"
- ✅ Countdown timer updates every second
- ✅ Format: "Xh Ym Zs" (e.g., "5h 23m 17s")
- ✅ Time accurately counts down to midnight UTC

**Actual Results**:
- [ ] Test not yet run

---

#### Test 4.4: Progress Bar Color Coding
**Steps**:
1. Use 35/50 Smart Compose calls
2. Use 15/20 Summarization calls
3. Use 9/10 Priority Ranking calls
4. Check Settings page

**Expected Results**:
- ✅ Smart Compose (70%): Green progress bar
- ✅ Summarization (75%): Yellow progress bar
- ✅ Priority Ranking (90%): Red progress bar
- ✅ Text colors match bar colors

**Actual Results**:
- [ ] Test not yet run

---

#### Test 4.5: Account Information
**Steps**:
1. Scroll to "Account" section
2. Observe displayed information

**Expected Results**:
- ✅ Email address displayed correctly
- ✅ Account ID shown in monospace font
- ✅ Matches logged-in user

**Actual Results**:
- [ ] Test not yet run

---

### 5. Integration Tests

#### Test 5.1: Multi-Feature Workflow
**Steps**:
1. Navigate to /priority
2. Click highest priority email
3. Click "Summarize"
4. Read summary
5. Click "Reply"
6. Start typing response
7. Accept Smart Compose suggestion
8. Send email
9. Navigate to /settings
10. Check quota usage

**Expected Results**:
- ✅ All features work together seamlessly
- ✅ Quota counters increment correctly:
  - Smart Compose: +1 (used in compose)
  - Summarization: +1 (used in email detail)
  - Priority Ranking: +1 (initial page load)
- ✅ No errors or crashes

**Actual Results**:
- [ ] Test not yet run

---

#### Test 5.2: Real-Time Updates
**Steps**:
1. Open /priority in browser tab 1
2. Open /settings in browser tab 2
3. In tab 1, click "Refresh Rankings"
4. Switch to tab 2
5. Observe quota update

**Expected Results**:
- ✅ Settings page reflects new usage immediately
- ✅ Progress bar updates
- ✅ Counter increments

**Actual Results**:
- [ ] Test not yet run

---

#### Test 5.3: Dark Mode Consistency
**Steps**:
1. Navigate to /settings
2. Toggle dark mode
3. Visit /priority
4. Compose new email with Smart Compose
5. Summarize an email

**Expected Results**:
- ✅ All AI features render correctly in dark mode
- ✅ Blue accent colors visible (not washed out)
- ✅ Ghost text readable in dark mode
- ✅ Progress bars visible
- ✅ Summary card readable

**Actual Results**:
- [ ] Test not yet run

---

### 6. Performance Tests

#### Test 6.1: Smart Compose Debounce
**Steps**:
1. Compose new email
2. Type rapidly without pausing
3. Observe API calls in browser Network tab

**Expected Results**:
- ✅ No API calls made while typing continuously
- ✅ API call triggers exactly 2 seconds after last keystroke
- ✅ Only one API call per typing session
- ✅ Debounce prevents excessive requests

**Actual Results**:
- [ ] Test not yet run

---

#### Test 6.2: Summary Caching
**Steps**:
1. Summarize an email
2. Hide summary
3. Show summary again
4. Check Network tab

**Expected Results**:
- ✅ First summarize: API call visible
- ✅ Second show: No new API call
- ✅ Summary loads instantly from memory
- ✅ Saves quota usage

**Actual Results**:
- [ ] Test not yet run

---

#### Test 6.3: Priority Ranking Performance
**Steps**:
1. Navigate to /priority with 50+ emails in inbox
2. Measure time to complete ranking
3. Check Network tab for API calls

**Expected Results**:
- ✅ Ranking completes in < 5 seconds
- ✅ Single API call (batch ranking)
- ✅ Loading indicator visible throughout
- ✅ No UI freeze or lag

**Actual Results**:
- [ ] Test not yet run

---

### 7. Edge Cases

#### Test 7.1: Empty Inbox
**Steps**:
1. Delete all emails in inbox
2. Navigate to /priority

**Expected Results**:
- ✅ "No emails to rank" message displayed
- ✅ No API call made
- ✅ No errors in console

**Actual Results**:
- [ ] Test not yet run

---

#### Test 7.2: Very Long Email
**Steps**:
1. Find or create email with 5000+ word body
2. Click "Summarize"

**Expected Results**:
- ✅ Summary generated successfully
- ✅ Respects 200 token output limit
- ✅ Captures main points despite length
- ✅ No timeout errors

**Actual Results**:
- [ ] Test not yet run

---

#### Test 7.3: Incomplete Email Composition
**Steps**:
1. Start composing email
2. Type only: "Hi"
3. Wait 2 seconds

**Expected Results**:
- ✅ No Smart Compose suggestion (text too short)
- ✅ No API call made
- ✅ Minimum 10 characters required

**Actual Results**:
- [ ] Test not yet run

---

#### Test 7.4: Rapid Feature Switching
**Steps**:
1. Navigate to /priority
2. Immediately click "Refresh Rankings"
3. Quickly navigate to /settings
4. Immediately navigate back to /priority

**Expected Results**:
- ✅ No race conditions
- ✅ Requests cancel properly
- ✅ UI remains responsive
- ✅ No duplicate quota charges

**Actual Results**:
- [ ] Test not yet run

---

## Summary of Results

### Test Execution Status
- **Total Tests**: 29
- **Passed**: 0
- **Failed**: 0
- **Skipped**: 0
- **Not Yet Run**: 29

### Critical Issues Found
- None yet

### Non-Critical Issues Found
- None yet

### Recommendations
- TBD after testing

---

## Testing Notes

**Environment Issues**:
- Fixed Card component import errors before testing

**API Configuration**:
- Backend running on localhost:3001
- OpenAI API key configured via .env
- Test account provisioned

**Browser Compatibility**:
- Primary testing: Google Chrome (latest)
- Additional testing needed: Firefox, Safari

---

## Next Steps

1. ✅ Fix Card component imports
2. ⏳ Execute all test cases systematically
3. ⏳ Document results for each test
4. ⏳ File bug reports for any failures
5. ⏳ Create user documentation based on successful tests
6. ⏳ Update PHASE5_COMPLETE.md with test results

---

**Test Execution Date**: December 16, 2025
**Tester**: Claude Sonnet 4.5
**Status**: In Progress
