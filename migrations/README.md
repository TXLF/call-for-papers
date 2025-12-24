# Database Schema

This directory contains SQLx migrations for the Call for Papers system.

## Overview

The database schema supports a complete conference management system with:
- User authentication (local and social providers)
- Talk submission and management
- Rating and review system
- Conference scheduling
- Email templates and logging

## Migrations

Migrations are applied automatically when the application starts.

### 20241224000001_create_users_and_auth.sql
Creates the user authentication system:
- `users` - User accounts (speakers and organizers)
- `auth_providers` - Social login providers (Google, Facebook, GitHub, Apple)
- `sessions` - Session management

### 20241224000002_create_talks_and_labels.sql
Creates the talk submission and rating system:
- `talks` - Talk proposals with title, summary, description, slides
- `labels` - Tags/categories for talks (manual and AI-generated)
- `talk_labels` - Many-to-many relationship between talks and labels
- `ratings` - Organizer ratings for talks (1-5 scale)

### 20241224000003_create_schedule.sql
Creates the conference scheduling system:
- `conferences` - Conference/event information
- `tracks` - Rooms or parallel tracks
- `schedule_slots` - Time slots with assigned talks

### 20241224000004_create_email_templates.sql
Creates the email communication system:
- `email_templates` - Customizable email templates
- `email_logs` - History of sent emails

## Schema Diagram

```
users
  ├─> auth_providers (multiple auth methods per user)
  ├─> sessions (active sessions)
  ├─> talks (as speaker)
  ├─> ratings (as organizer)
  └─> talk_labels.added_by (who added label)

talks
  ├─> talk_labels (many-to-many with labels)
  ├─> ratings (multiple organizer ratings)
  └─> schedule_slots (scheduled time)

conferences
  ├─> tracks (rooms/parallel tracks)
  ├─> schedule_slots (conference schedule)
  └─> email_templates (conference-specific templates)
```

## Talk States

- `submitted` - Initial state when speaker submits
- `pending` - Accepted by organizers, awaiting speaker confirmation
- `accepted` - Speaker confirmed they will present
- `rejected` - Not selected for the conference

## Auth Provider Types

- `local` - Username/password authentication
- `google` - Google OAuth
- `facebook` - Facebook OAuth
- `github` - GitHub OAuth
- `apple` - Apple Sign In

## Email Template Types

- `submission_confirmation` - Sent when talk is submitted
- `talk_accepted` - Sent when talk is accepted
- `talk_rejected` - Sent when talk is rejected
- `talk_pending` - Sent when awaiting speaker confirmation
- `schedule_notification` - Sent with schedule information
- `custom` - Custom templates for other purposes

## Development

To run migrations manually:
```bash
sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers"
```

To create a new migration:
```bash
sqlx migrate add <migration_name>
```

To revert the last migration:
```bash
sqlx migrate revert --database-url "postgres://postgres:postgres@localhost/call_for_papers"
```
