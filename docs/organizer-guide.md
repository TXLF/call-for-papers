# Organizer Guide

This guide provides comprehensive documentation for conference organizers using the Call for Papers system. As an organizer, you have access to powerful tools for managing submissions, rating talks, building schedules, and communicating with speakers.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Dashboard Overview](#dashboard-overview)
3. [Managing Talk Submissions](#managing-talk-submissions)
4. [Rating System](#rating-system)
5. [Label Management](#label-management)
6. [AI-Powered Features](#ai-powered-features)
7. [Schedule Management](#schedule-management)
8. [Communication Tools](#communication-tools)
9. [Data Export](#data-export)
10. [Configuration](#configuration)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Accessing Organizer Features

1. **Create an Account**: Sign up using username/password or social login (Google, GitHub, Apple, Facebook, LinkedIn)
2. **Organizer Role**: Contact your system administrator to have the organizer role assigned to your account
3. **Login**: Once you have organizer privileges, additional menu items will appear in the navigation

### Organizer Navigation Menu

After logging in as an organizer, you'll see these additional menu items:

- **Dashboard** - Overview of submission statistics
- **Review Talks** - View and manage all submissions
- **Manage Labels** - Create and organize labels/tags
- **Manage Tracks** - Define conference rooms/tracks
- **Manage Time Slots** - Create schedule time slots
- **Assign Talks** - Assign accepted talks to time slots
- **Schedule Builder** - Visual schedule building interface
- **Ratings Dashboard** - View rating statistics
- **Email Templates** - Create email templates
- **Bulk Email** - Send emails to multiple speakers
- **Export Talks** - Export submission data
- **AI Auto-Tag** - Automatically tag submissions with AI
- **Configuration** - View system configuration

---

## Dashboard Overview

The **Organizer Dashboard** (`/organizer/dashboard`) provides a quick overview of your conference:

### Statistics Displayed

- **Total Submissions**: Number of talks submitted
- **Submitted**: Talks in submitted state
- **Pending**: Talks accepted by organizers, awaiting speaker confirmation
- **Accepted**: Talks confirmed by speakers
- **Rejected**: Talks not selected

### What to Do

- Review statistics to understand submission volume
- Check for talks requiring action (pending confirmations)
- Monitor the overall health of your CFP process

---

## Managing Talk Submissions

### Viewing All Submissions

Navigate to **Review Talks** (`/organizer/talks`) to see all submitted talks.

### Talk Details

Each talk shows:

- **Title**: Talk title
- **Speaker**: Name and email
- **Status**: Current state (submitted, pending, accepted, rejected)
- **Short Summary**: Brief description
- **Long Description**: Detailed description (if provided)
- **Slides**: Uploaded slides (if available)
- **Labels**: Tags applied to the talk
- **Ratings**: Average rating and number of ratings

### Filtering and Searching

Use the available filters to:

- Filter by status (submitted, pending, accepted, rejected)
- Filter by label
- Filter by track
- Search by title or speaker name

### Changing Talk State

As an organizer, you can move talks through these states:

1. **Submitted → Pending**: Accept a talk (sends notification to speaker)
2. **Submitted → Rejected**: Decline a talk
3. **Pending → Accepted**: Speaker has confirmed
4. **Pending → Rejected**: Speaker declined or you revoked acceptance
5. **Accepted → Rejected**: Cancel an accepted talk (use with caution)

**To change state:**

1. Click on a talk to view details
2. Select the new state from the dropdown
3. Click "Update Status"
4. Speaker will be notified automatically (if email is configured)

### Adding/Removing Labels

You can tag any talk with labels:

1. View talk details
2. Click "Add Label"
3. Select labels from the list
4. Click "Remove" to remove a label

Labels help organize and categorize submissions for easier management.

---

## Rating System

### Rating Talks

The rating system helps organizers collaboratively evaluate submissions:

1. Navigate to **Review Talks**
2. Click on a talk to view details
3. Scroll to the "Rate This Talk" section
4. Select a rating (1-5 stars):
   - 1 star: Poor fit
   - 2 stars: Below average
   - 3 stars: Average
   - 4 stars: Good
   - 5 stars: Excellent
5. Optionally add comments
6. Click "Submit Rating"

### Your Rating

You can:

- **Update**: Change your rating at any time
- **Delete**: Remove your rating completely
- **Comment**: Add notes for other organizers (visible to all organizers)

### Viewing Ratings

**Individual Talk Ratings:**

- View all ratings on the talk detail page
- See who rated and their scores
- Read rating comments

**Ratings Dashboard:**

Navigate to **Ratings Dashboard** (`/organizer/ratings`) to see:

- **Overall Statistics**: Average ratings across all talks
- **Rating Distribution**: How many talks have each rating level
- **Top Rated Talks**: Highest-rated submissions
- **Unrated Talks**: Submissions needing review
- **Your Rating Activity**: How many talks you've rated

### Best Practices for Rating

- Rate consistently across all submissions
- Use the full rating scale (don't just use 3-5)
- Add comments to explain ratings below 3 or above 4
- Rate independently before discussing with other organizers
- Review all submissions before making final decisions

---

## Label Management

Labels help categorize and organize submissions. Navigate to **Manage Labels** (`/organizer/labels`).

### Creating Labels

1. Click "Create New Label"
2. Enter label name (e.g., "Security", "Beginner-Friendly", "Keynote")
3. Choose a color for visual identification
4. Optionally add a description
5. Click "Create"

### Editing Labels

1. Find the label in the list
2. Click "Edit"
3. Update name, color, or description
4. Click "Save"

### Deleting Labels

1. Find the label in the list
2. Click "Delete"
3. Confirm deletion
4. Note: This removes the label from all talks

### Label Organization Tips

Create labels for:

- **Topics**: Security, DevOps, Cloud, AI/ML, Web Development
- **Audience Level**: Beginner, Intermediate, Advanced
- **Talk Type**: Technical Deep Dive, Case Study, Tutorial, Panel
- **Special Categories**: Keynote, Lightning Talk, Workshop
- **Internal Tags**: Staff Pick, Community Favorite, Needs Review

---

## AI-Powered Features

The system integrates with Claude and OpenAI APIs for automated analysis.

### Auto-Tagging with AI

Navigate to **AI Auto-Tag** (`/organizer/ai-auto-tag`).

**This feature:**

- Analyzes all submitted talks using AI
- Suggests relevant labels based on content
- Can automatically create and apply labels
- Saves time on manual categorization

**To use:**

1. Ensure Claude or OpenAI API is configured (see Configuration)
2. Click "Analyze with Claude" or "Analyze with OpenAI"
3. Review suggested labels
4. Select which suggestions to apply
5. Click "Apply Selected Labels"

**What the AI analyzes:**

- Talk title
- Short and long descriptions
- Technical topics mentioned
- Audience level indicators
- Talk type/format

### Creating AI-Suggested Labels

The AI can suggest new labels based on submission content:

1. Go to **AI Auto-Tag**
2. Click "Suggest New Labels"
3. AI analyzes all submissions and suggests label categories
4. Review suggestions
5. Create labels you want to use
6. Run auto-tagging again to apply them

### Exporting for External AI Analysis

If you want to analyze submissions in ChatGPT or Claude directly:

1. Navigate to **Export Talks** (`/organizer/export`)
2. Select format: CSV or JSON
3. Click "Export"
4. Upload the file to your AI tool of choice
5. Ask the AI to analyze, categorize, or provide recommendations

---

## Schedule Management

Build your conference schedule through these steps:

### 1. Define Conference Tracks

Navigate to **Manage Tracks** (`/organizer/tracks`).

Tracks represent different rooms or parallel sessions.

**To create a track:**

1. Click "Create New Track"
2. Enter track name (e.g., "Main Hall", "Room A", "Workshop Space")
3. Optionally add description and capacity
4. Click "Create"

**To edit or delete:**

- Click "Edit" to update track details
- Click "Delete" to remove (only if no talks assigned)

### 2. Create Time Slots

Navigate to **Manage Time Slots** (`/organizer/schedule-slots`).

Time slots define when talks can occur.

**To create a time slot:**

1. Click "Create New Slot"
2. Select date and time
3. Set duration (e.g., 45 minutes, 1 hour)
4. Select track (room)
5. Optionally add notes (e.g., "Opening Keynote")
6. Click "Create"

**Tips:**

- Create slots for all days of your conference
- Account for breaks, lunch, and networking time
- Create slots across all tracks
- Consider different talk lengths (keynotes vs lightning talks)

### 3. Assign Talks to Slots

Navigate to **Assign Talks** (`/organizer/assign-talks`).

**To assign a talk:**

1. View available time slots
2. See list of accepted talks
3. Drag and drop a talk to a slot (or use "Assign" button)
4. Confirm assignment

**To unassign a talk:**

1. Find the assigned slot
2. Click "Unassign"
3. Talk returns to unassigned pool

### 4. Schedule Builder

Navigate to **Schedule Builder** (`/organizer/schedule-builder`) for a visual interface.

**Features:**

- Calendar view of entire schedule
- See all tracks side-by-side
- Drag-and-drop interface
- Identify scheduling conflicts
- Export schedule view

### 5. Public Schedule

Speakers and attendees can view the published schedule at `/schedule`.

This page shows:

- All scheduled talks with times and locations
- Talk titles, speakers, and descriptions
- Track/room assignments
- Conference days organized

---

## Communication Tools

### Email Templates

Navigate to **Email Templates** (`/organizer/email-templates`).

Create reusable email templates for common communications.

**To create a template:**

1. Click "Create New Template"
2. Enter template name (e.g., "Acceptance Email", "Rejection Email")
3. Write subject line
4. Write email body
5. Use placeholders for personalization:
   - `{{speaker_name}}` - Speaker's name
   - `{{talk_title}}` - Talk title
   - `{{conference_name}}` - Conference name
   - `{{acceptance_deadline}}` - Deadline to confirm
6. Click "Save"

**To edit or delete:**

- Click "Edit" to update template
- Click "Delete" to remove

**Template Ideas:**

- **Acceptance Email**: Congratulate speaker, ask for confirmation
- **Rejection Email**: Thank speaker, encourage future submissions
- **Reminder Email**: Remind speaker about deadlines
- **Schedule Notification**: Inform speaker of their time slot
- **Slide Request**: Request slides before conference

### Bulk Email

Navigate to **Bulk Email** (`/organizer/bulk-email`).

Send emails to multiple speakers at once.

**To send bulk email:**

1. Select template (or write custom email)
2. Choose recipients:
   - All speakers
   - Speakers with accepted talks
   - Speakers with pending talks
   - Speakers with rejected talks
   - Custom selection
3. Preview email with sample data
4. Click "Send"

**Notes:**

- Emails are sent individually (not CC/BCC)
- Placeholders are personalized for each recipient
- Check your SMTP configuration before sending
- Test with a small group first

### Automatic Notifications

The system automatically sends emails when:

- Talk status changes (if email is configured)
- Speaker confirms or declines acceptance
- You can disable auto-notifications in configuration

---

## Data Export

Navigate to **Export Talks** (`/organizer/export`).

### Export Formats

**CSV (Comma-Separated Values):**

- Compatible with Excel, Google Sheets
- Good for data analysis and reporting
- Includes all talk fields and metadata

**JSON (JavaScript Object Notation):**

- Compatible with AI tools and APIs
- Structured data format
- Good for programmatic analysis

### What's Exported

- Talk ID
- Title
- Speaker name and email
- Short summary
- Long description
- Status (submitted, pending, accepted, rejected)
- Labels/tags
- Ratings (average and count)
- Submission date
- Track assignment (if scheduled)
- Time slot (if scheduled)

### Use Cases

- **AI Analysis**: Upload to ChatGPT/Claude for insights
- **Reporting**: Generate statistics and reports
- **Backup**: Keep offline copy of submissions
- **Integration**: Import into other tools
- **Board Review**: Share with conference committee

---

## Configuration

Navigate to **Configuration** (`/organizer/configuration`).

View current system configuration including:

### Conference Information

- Conference name and short name
- Year
- Location (city, state, country)
- Dates
- Website URL

### Features

- Speaker registration enabled/disabled
- Ratings enabled/disabled
- AI features enabled/disabled
- Schedule public/private

### Branding

- Primary and secondary colors
- Logo URL
- Favicon URL
- Custom CSS

### Submission Settings

- Minimum/maximum title length
- Minimum/maximum summary length
- Allowed slide formats
- Maximum file size

### Email Settings

- SMTP configured/not configured
- From email address
- Auto-notification settings

### Integrations

- Google OAuth: Configured / Not Configured
- GitHub OAuth: Configured / Not Configured
- Apple OAuth: Configured / Not Configured
- Facebook OAuth: Configured / Not Configured
- LinkedIn OAuth: Configured / Not Configured
- Claude API: Configured / Not Configured
- OpenAI API: Configured / Not Configured
- SMTP Email: Configured / Not Configured

### Security

- JWT token expiration
- Rate limiting settings
- Password requirements

### Modifying Configuration

Configuration is currently **read-only** in the UI. To modify:

1. Edit `config.toml` file on the server
2. Restart the application
3. Changes will be reflected in the Configuration page

See the deployment guide for detailed configuration instructions.

---

## Best Practices

### Before CFP Opens

1. **Configure the system**: Set up branding, emails, integrations
2. **Create labels**: Define categories and tags you'll use
3. **Define tracks**: Set up conference rooms/spaces
4. **Test submission**: Submit a test talk to verify everything works
5. **Create email templates**: Prepare acceptance/rejection templates
6. **Set up organizer accounts**: Ensure all reviewers have access

### During CFP Period

1. **Monitor submissions**: Check dashboard regularly
2. **Respond promptly**: Answer speaker questions quickly
3. **Tag submissions**: Apply labels as talks come in
4. **Review early**: Start rating talks as they're submitted
5. **Communicate**: Send acknowledgment emails
6. **Extend deadline**: If needed, communicate changes clearly

### Review and Selection Phase

1. **Rate independently**: Have organizers rate before discussing
2. **Use AI suggestions**: Auto-tag to identify patterns
3. **Review ratings dashboard**: Identify top-rated talks
4. **Discuss as team**: Meet to make final decisions
5. **Check diversity**: Ensure variety in topics, speakers, levels
6. **Make decisions**: Move talks to pending or rejected
7. **Send notifications**: Use bulk email for acceptances/rejections
8. **Set deadlines**: Give speakers clear timeline to confirm

### Schedule Building Phase

1. **Wait for confirmations**: Only schedule accepted talks
2. **Create time slots**: Define all slots before assigning
3. **Consider constraints**: Speaker availability, topic conflicts
4. **Use schedule builder**: Visual interface helps spot issues
5. **Balance tracks**: Distribute popular topics across rooms
6. **Allow breaks**: Don't pack schedule too tightly
7. **Review conflicts**: Check for speaker or topic overlap
8. **Publish schedule**: Make public when finalized

### Before Conference

1. **Request slides**: Send reminder emails with deadline
2. **Confirm attendance**: Follow up with speakers
3. **Send schedule details**: Remind speakers of their time/room
4. **Test AV requirements**: Confirm technical needs
5. **Prepare backup plans**: Have contingency for cancellations

### After Conference

1. **Thank speakers**: Send appreciation emails
2. **Collect feedback**: Survey speakers and attendees
3. **Archive data**: Export all data for records
4. **Document process**: Note what worked and what didn't
5. **Plan improvements**: Use learnings for next year

---

## Troubleshooting

### Common Issues

#### Can't See Organizer Menu Items

**Problem**: Logged in but don't see organizer features.

**Solution**:
- Verify you have organizer role (check with admin)
- Log out and log back in
- Clear browser cache
- Check system configuration allows organizers

#### Email Notifications Not Sending

**Problem**: Status changes don't trigger emails.

**Solution**:
- Check SMTP configuration in Configuration page
- Verify "SMTP Email: Configured" shows as enabled
- Check server logs for email errors
- Test with bulk email first
- Contact system administrator

#### AI Features Not Working

**Problem**: Auto-tag or AI suggestions fail.

**Solution**:
- Check Configuration page shows AI APIs as "Configured"
- Verify API keys are valid (check with admin)
- Check you have available API credits
- Try alternative AI provider (Claude vs OpenAI)
- Review error messages in browser console

#### Can't Assign Talk to Slot

**Problem**: Assignment fails or talk doesn't appear in slot.

**Solution**:
- Ensure talk is in "Accepted" status (not pending)
- Check time slot exists and is correct track
- Verify slot isn't already assigned to another talk
- Check for date/time conflicts
- Refresh page and try again

#### Ratings Not Appearing

**Problem**: Submitted ratings don't show up.

**Solution**:
- Check ratings are enabled in Configuration
- Verify you have organizer role
- Refresh page
- Check network tab for API errors
- Try different browser

#### Export Download Fails

**Problem**: Export button doesn't download file.

**Solution**:
- Check browser popup blocker
- Try different export format (CSV vs JSON)
- Check you have talks to export
- Try different browser
- Check disk space on your computer

### Getting Help

If you encounter issues not covered here:

1. **Check Configuration**: Review Configuration page for missing setup
2. **Check Browser Console**: Look for JavaScript errors (F12 → Console)
3. **Check Network Requests**: Look for failed API calls (F12 → Network)
4. **Try Different Browser**: Rule out browser-specific issues
5. **Contact Administrator**: Provide details about the issue and any error messages
6. **Check GitHub Issues**: See if others have reported similar problems
7. **Submit Bug Report**: Use the bug report template if you found a bug

### Performance Tips

For large numbers of submissions (500+):

- Use filters and search instead of scrolling
- Rate in batches rather than all at once
- Export data for analysis in spreadsheet tools
- Use AI auto-tag instead of manual labeling
- Clear browser cache if interface becomes slow

---

## Appendix: Keyboard Shortcuts

**General Navigation:**

- `Ctrl/Cmd + K` - Quick search (if enabled)

**Talk Review:**

- `←` / `→` - Navigate between talks
- `R` - Rate current talk
- `A` - Accept talk
- `Esc` - Close modal/dialog

**Schedule Builder:**

- Click and drag to assign talks
- `Ctrl/Cmd + Z` - Undo last assignment
- `Delete` - Unassign selected talk

Note: Keyboard shortcuts may vary based on your browser and operating system.

---

## Need More Help?

- **Speaker Guide**: See [docs/speaker-guide.md](speaker-guide.md) for speaker instructions
- **Development Guide**: See [DEVELOPMENT.md](../DEVELOPMENT.md) for technical setup
- **Configuration Guide**: See [config.example.toml](../config.example.toml) for all configuration options
- **API Documentation**: See [API.md](API.md) (if available) for programmatic access
- **GitHub Repository**: https://github.com/TXLF/call-for-papers
- **Report Issues**: https://github.com/TXLF/call-for-papers/issues

---

**Last Updated**: December 2025
**Version**: 1.0
**Maintained by**: TXLF Contributors
