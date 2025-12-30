# Call for Papers - API Documentation

This document provides comprehensive API documentation for the Call for Papers REST API.

## Table of Contents

1. [Overview](#overview)
2. [Base URL](#base-url)
3. [Authentication](#authentication)
4. [Authorization](#authorization)
5. [Rate Limiting](#rate-limiting)
6. [Error Handling](#error-handling)
7. [API Endpoints](#api-endpoints)
   - [Health Check](#health-check)
   - [Authentication](#authentication-endpoints)
   - [Talks](#talk-endpoints)
   - [Labels](#label-endpoints)
   - [Ratings](#rating-endpoints)
   - [Conferences](#conference-endpoints)
   - [Tracks](#track-endpoints)
   - [Schedule Slots](#schedule-slot-endpoints)
   - [Schedule](#schedule-endpoints)
   - [Email Templates](#email-template-endpoints)
   - [Bulk Email](#bulk-email-endpoints)
   - [Export](#export-endpoints)
   - [AI Tagging](#ai-tagging-endpoints)
   - [Dashboard](#dashboard-endpoints)
   - [Configuration](#configuration-endpoints)
8. [Integration Examples](#integration-examples)
9. [Webhook Events](#webhook-events)
10. [Changelog](#changelog)

---

## Overview

The Call for Papers API is a REST API that provides programmatic access to manage conference talk submissions, ratings, scheduling, and communication.

**API Characteristics:**
- **Protocol**: HTTPS (HTTP in development)
- **Format**: JSON
- **Authentication**: JWT (JSON Web Tokens)
- **Versioning**: Currently v1 (implicit, may add explicit versioning in future)

---

## Base URL

```
Production: https://your-domain.com/api
Development: http://localhost:8080/api
```

All endpoints are prefixed with `/api`.

---

## Authentication

### Authentication Methods

The API supports two authentication methods:

1. **Username/Password**: Traditional credential-based authentication
2. **OAuth 2.0**: Social login via Google, GitHub, Apple, Facebook, LinkedIn

### Obtaining a JWT Token

**Method 1: Login with Credentials**

```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "full_name": "John Doe",
    "is_organizer": false
  }
}
```

**Method 2: OAuth 2.0**

1. Redirect user to `/api/auth/{provider}` (e.g., `/api/auth/google`)
2. User authenticates with provider
3. Provider redirects to `/api/auth/{provider}/callback`
4. Frontend receives JWT token

### Using the JWT Token

Include the JWT token in the `Authorization` header:

```bash
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token Lifecycle

- **Expiration**: 7 days (168 hours) by default
- **Renewal**: Re-authenticate when token expires
- **Storage**: Store securely (httpOnly cookie or secure storage)

---

## Authorization

The API uses role-based access control (RBAC) with two roles:

### Public

No authentication required. Accessible by anyone.

**Endpoints:**
- Health checks
- Authentication endpoints
- Public schedule view
- List public conferences, tracks, labels

### Authenticated

Requires valid JWT token. Accessible by any logged-in user.

**Endpoints:**
- Submit talks
- View own talks
- Upload slides
- Respond to talk acceptances
- Manage own labels on talks

### Organizer

Requires valid JWT token AND `is_organizer: true` flag.

**Endpoints:**
- Review all talks
- Rate talks
- Change talk states
- Manage labels, tracks, schedules
- Send bulk emails
- Export data
- AI features
- Dashboard statistics

---

## Rate Limiting

Rate limiting is enforced at the reverse proxy level (Envoy/Nginx).

**Default Limits:**
- **Public endpoints**: 100 requests/minute per IP
- **Authentication endpoints**: 20 requests/minute per IP
- **Authenticated endpoints**: 1000 requests/minute per user

**Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1672531200
```

**429 Response:**
```json
{
  "error": "Rate limit exceeded. Try again in 60 seconds."
}
```

---

## Error Handling

### HTTP Status Codes

- `200 OK`: Success
- `201 Created`: Resource created
- `204 No Content`: Success with no response body
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Authentication required or invalid
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (duplicate)
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service temporarily unavailable

### Error Response Format

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": {
    "field": "Additional context"
  }
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `AUTH_REQUIRED` | Authentication required |
| `INVALID_TOKEN` | JWT token is invalid or expired |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `VALIDATION_ERROR` | Request validation failed |
| `DUPLICATE` | Resource already exists |
| `STATE_ERROR` | Invalid state transition |
| `DATABASE_ERROR` | Database operation failed |

---

## API Endpoints

### Health Check

#### Check API Health

**Endpoint:** `GET /api/health`

**Auth:** None

**Description:** Basic health check to verify API is responding.

**Response:**
```
OK
```

#### Check Database Health

**Endpoint:** `GET /api/health/db`

**Auth:** None

**Description:** Health check including database connectivity.

**Response:**
```
OK
```

**Error Response (503):**
```json
{
  "error": "Database error: connection refused"
}
```

---

### Authentication Endpoints

#### Register User

**Endpoint:** `POST /api/auth/register`

**Auth:** None

**Description:** Create a new user account.

**Request:**
```json
{
  "email": "user@example.com",
  "username": "johndoe",
  "password": "SecurePassword123!",
  "full_name": "John Doe",
  "bio": "Software engineer and conference speaker"
}
```

**Response (201):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "full_name": "John Doe",
    "bio": "Software engineer and conference speaker",
    "is_organizer": false,
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

**Validation:**
- `email`: Required, valid email format, unique
- `username`: Optional, alphanumeric + underscore, unique
- `password`: Required, minimum 8 characters
- `full_name`: Required, 1-255 characters
- `bio`: Optional, max 1000 characters

#### Login

**Endpoint:** `POST /api/auth/login`

**Auth:** None

**Description:** Authenticate with email and password.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "full_name": "John Doe",
    "is_organizer": false
  }
}
```

**Error Response (401):**
```json
{
  "error": "Invalid credentials"
}
```

#### OAuth: Google

**Endpoint:** `GET /api/auth/google`

**Auth:** None

**Description:** Initiate Google OAuth flow. Redirects to Google login.

**Query Parameters:**
- `redirect_uri` (optional): Frontend URL to redirect after authentication

**Response:** HTTP 302 redirect to Google

---

**Endpoint:** `GET /api/auth/google/callback`

**Auth:** None (OAuth code in query)

**Description:** Google OAuth callback. Exchanges code for user token.

**Query Parameters:**
- `code`: OAuth authorization code from Google
- `state`: CSRF protection token

**Response:** HTTP 302 redirect to frontend with token

#### OAuth: GitHub

**Endpoint:** `GET /api/auth/github`

**Endpoint:** `GET /api/auth/github/callback`

Similar to Google OAuth.

#### OAuth: Apple

**Endpoint:** `GET /api/auth/apple`

**Endpoint:** `GET /api/auth/apple/callback`

Similar to Google OAuth.

#### OAuth: Facebook

**Endpoint:** `GET /api/auth/facebook`

**Endpoint:** `GET /api/auth/facebook/callback`

Similar to Google OAuth.

#### OAuth: LinkedIn

**Endpoint:** `GET /api/auth/linkedin`

**Endpoint:** `GET /api/auth/linkedin/callback`

Similar to Google OAuth.

---

### Talk Endpoints

#### Create Talk

**Endpoint:** `POST /api/talks`

**Auth:** Required (Authenticated)

**Description:** Submit a new talk proposal.

**Request:**
```json
{
  "title": "Introduction to Rust for Systems Programming",
  "short_summary": "Learn the basics of Rust and how to build fast, safe systems software.",
  "long_description": "This talk will cover Rust's ownership model, memory safety guarantees, and practical examples of building systems software. We'll explore real-world use cases and compare Rust to C and C++.",
  "label_ids": [
    "label-uuid-1",
    "label-uuid-2"
  ]
}
```

**Response (201):**
```json
{
  "id": "talk-uuid",
  "speaker_id": "user-uuid",
  "title": "Introduction to Rust for Systems Programming",
  "short_summary": "Learn the basics of Rust...",
  "long_description": "This talk will cover...",
  "slides_url": null,
  "state": "submitted",
  "submitted_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z",
  "labels": [
    {
      "id": "label-uuid-1",
      "name": "Systems Programming",
      "color": "#ff5722"
    }
  ]
}
```

**Validation:**
- `title`: Required, 10-500 characters
- `short_summary`: Required, 50-1000 characters
- `long_description`: Optional, max 5000 characters
- `label_ids`: Optional array of label UUIDs

#### List My Talks

**Endpoint:** `GET /api/talks/mine`

**Auth:** Required (Authenticated)

**Description:** Get all talks submitted by the authenticated user.

**Response (200):**
```json
{
  "talks": [
    {
      "id": "talk-uuid",
      "title": "Introduction to Rust",
      "short_summary": "Learn the basics...",
      "state": "submitted",
      "submitted_at": "2025-01-15T10:30:00Z",
      "labels": [...]
    }
  ],
  "total": 3
}
```

#### Get Talk

**Endpoint:** `GET /api/talks/:id`

**Auth:** Required (Authenticated)

**Description:** Get a specific talk by ID. Users can only view their own talks unless they are organizers.

**Response (200):**
```json
{
  "id": "talk-uuid",
  "speaker_id": "user-uuid",
  "speaker": {
    "id": "user-uuid",
    "full_name": "John Doe",
    "email": "john@example.com"
  },
  "title": "Introduction to Rust",
  "short_summary": "Learn the basics...",
  "long_description": "This talk will cover...",
  "slides_url": "/uploads/talk-uuid-slides.pdf",
  "state": "submitted",
  "submitted_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z",
  "labels": [...],
  "average_rating": null,
  "rating_count": 0
}
```

#### Update Talk

**Endpoint:** `PUT /api/talks/:id`

**Auth:** Required (Authenticated, Own Talk)

**Description:** Update a talk. Only allowed before CFP deadline and while in 'submitted' or 'pending' state.

**Request:**
```json
{
  "title": "Advanced Rust for Systems Programming",
  "short_summary": "Updated summary...",
  "long_description": "Updated description..."
}
```

**Response (200):**
```json
{
  "id": "talk-uuid",
  "title": "Advanced Rust for Systems Programming",
  ...
}
```

#### Delete Talk

**Endpoint:** `DELETE /api/talks/:id`

**Auth:** Required (Authenticated, Own Talk)

**Description:** Delete a talk. Only allowed in 'submitted' or 'rejected' state.

**Response (204):** No content

#### Upload Slides

**Endpoint:** `POST /api/talks/:id/upload-slides`

**Auth:** Required (Authenticated, Own Talk)

**Description:** Upload presentation slides for a talk.

**Request:** `multipart/form-data`
- `file`: Slide file (PDF, PPTX, KEY, ODP)
- Max size: 50 MB

**Response (200):**
```json
{
  "slides_url": "/uploads/550e8400-slides.pdf",
  "message": "Slides uploaded successfully"
}
```

**Supported formats:** `.pdf`, `.pptx`, `.key`, `.odp`

#### Respond to Talk

**Endpoint:** `POST /api/talks/:id/respond`

**Auth:** Required (Authenticated, Own Talk)

**Description:** Accept or decline a talk that has been moved to 'pending' state by organizers.

**Request:**
```json
{
  "accept": true
}
```

**Response (200):**
```json
{
  "id": "talk-uuid",
  "state": "accepted",
  "message": "Talk accepted successfully"
}
```

**Notes:**
- `accept: true` → state changes to "accepted"
- `accept: false` → state changes to "rejected"
- Only works when talk is in "pending" state

#### List All Talks (Organizer)

**Endpoint:** `GET /api/talks`

**Auth:** Required (Organizer)

**Description:** List all submitted talks with filtering and pagination.

**Query Parameters:**
- `state`: Filter by state (`submitted`, `pending`, `accepted`, `rejected`)
- `label_id`: Filter by label UUID
- `track_id`: Filter by track UUID
- `limit`: Results per page (default: 50, max: 100)
- `offset`: Page offset (default: 0)
- `sort`: Sort field (`submitted_at`, `updated_at`, `rating`)
- `order`: Sort order (`asc`, `desc`)

**Example:**
```
GET /api/talks?state=submitted&limit=20&offset=0&sort=submitted_at&order=desc
```

**Response (200):**
```json
{
  "talks": [
    {
      "id": "talk-uuid",
      "speaker": {
        "id": "user-uuid",
        "full_name": "John Doe",
        "email": "john@example.com"
      },
      "title": "Introduction to Rust",
      "short_summary": "Learn the basics...",
      "state": "submitted",
      "submitted_at": "2025-01-15T10:30:00Z",
      "labels": [...],
      "average_rating": 4.5,
      "rating_count": 10
    }
  ],
  "total": 156,
  "limit": 20,
  "offset": 0
}
```

#### Change Talk State (Organizer)

**Endpoint:** `PUT /api/talks/:id/state`

**Auth:** Required (Organizer)

**Description:** Change the state of a talk.

**Request:**
```json
{
  "state": "pending"
}
```

**Valid transitions:**
- `submitted` → `pending` (accept talk)
- `submitted` → `rejected` (reject talk)
- `pending` → `accepted` (speaker confirmed)
- `pending` → `rejected` (speaker declined or organizer revoked)
- `accepted` → `rejected` (cancel accepted talk)

**Response (200):**
```json
{
  "id": "talk-uuid",
  "state": "pending",
  "message": "Talk state updated successfully"
}
```

---

### Label Endpoints

#### List Labels

**Endpoint:** `GET /api/labels`

**Auth:** None

**Description:** Get all available labels.

**Response (200):**
```json
{
  "labels": [
    {
      "id": "label-uuid",
      "name": "Systems Programming",
      "description": "Low-level systems, OS, embedded",
      "color": "#ff5722",
      "is_ai_generated": false,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 25
}
```

#### Create Label (Organizer)

**Endpoint:** `POST /api/labels`

**Auth:** Required (Organizer)

**Description:** Create a new label.

**Request:**
```json
{
  "name": "Cloud Native",
  "description": "Kubernetes, Docker, microservices",
  "color": "#2196f3"
}
```

**Response (201):**
```json
{
  "id": "label-uuid",
  "name": "Cloud Native",
  "description": "Kubernetes, Docker, microservices",
  "color": "#2196f3",
  "is_ai_generated": false,
  "created_at": "2025-01-15T10:30:00Z"
}
```

**Validation:**
- `name`: Required, unique, 1-100 characters
- `description`: Optional, max 500 characters
- `color`: Optional, valid hex color (e.g., `#ff5722`)

#### Update Label (Organizer)

**Endpoint:** `PUT /api/labels/:id`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "name": "Cloud Native & DevOps",
  "description": "Updated description",
  "color": "#1976d2"
}
```

**Response (200):**
```json
{
  "id": "label-uuid",
  "name": "Cloud Native & DevOps",
  ...
}
```

#### Delete Label (Organizer)

**Endpoint:** `DELETE /api/labels/:id`

**Auth:** Required (Organizer)

**Description:** Delete a label. Removes it from all talks.

**Response (204):** No content

#### Get Talk Labels

**Endpoint:** `GET /api/talks/:id/labels`

**Auth:** Required (Authenticated)

**Description:** Get all labels for a specific talk.

**Response (200):**
```json
{
  "labels": [
    {
      "id": "label-uuid",
      "name": "Systems Programming",
      "color": "#ff5722",
      "added_by": {
        "id": "user-uuid",
        "full_name": "Jane Organizer"
      },
      "added_at": "2025-01-15T11:00:00Z"
    }
  ]
}
```

#### Add Labels to Talk

**Endpoint:** `POST /api/talks/:id/labels`

**Auth:** Required (Authenticated)

**Description:** Add one or more labels to a talk.

**Request:**
```json
{
  "label_ids": [
    "label-uuid-1",
    "label-uuid-2"
  ]
}
```

**Response (200):**
```json
{
  "message": "Labels added successfully",
  "labels": [...]
}
```

#### Remove Label from Talk

**Endpoint:** `DELETE /api/talks/:id/labels/:label_id`

**Auth:** Required (Authenticated)

**Description:** Remove a label from a talk.

**Response (204):** No content

---

### Rating Endpoints

#### Rate Talk (Organizer)

**Endpoint:** `POST /api/talks/:id/rate`

**Auth:** Required (Organizer)

**Description:** Create or update a rating for a talk.

**Request:**
```json
{
  "rating": 5,
  "notes": "Excellent proposal with clear objectives and strong speaker expertise."
}
```

**Validation:**
- `rating`: Required, integer 1-5
- `notes`: Optional, max 1000 characters

**Response (200):**
```json
{
  "id": "rating-uuid",
  "talk_id": "talk-uuid",
  "organizer_id": "user-uuid",
  "rating": 5,
  "notes": "Excellent proposal...",
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

#### Get Talk Ratings (Organizer)

**Endpoint:** `GET /api/talks/:id/ratings`

**Auth:** Required (Organizer)

**Description:** Get all ratings for a talk.

**Response (200):**
```json
{
  "ratings": [
    {
      "id": "rating-uuid",
      "organizer": {
        "id": "user-uuid",
        "full_name": "Jane Organizer"
      },
      "rating": 5,
      "notes": "Excellent proposal...",
      "created_at": "2025-01-15T10:30:00Z"
    }
  ],
  "average": 4.5,
  "count": 10
}
```

#### Get My Rating (Organizer)

**Endpoint:** `GET /api/talks/:id/rate/mine`

**Auth:** Required (Organizer)

**Description:** Get the authenticated organizer's rating for a talk.

**Response (200):**
```json
{
  "id": "rating-uuid",
  "rating": 5,
  "notes": "Excellent proposal...",
  "created_at": "2025-01-15T10:30:00Z"
}
```

**Response (404):** If no rating exists yet

#### Delete Rating (Organizer)

**Endpoint:** `DELETE /api/talks/:id/rate`

**Auth:** Required (Organizer)

**Description:** Delete the authenticated organizer's rating for a talk.

**Response (204):** No content

#### Get Ratings Statistics (Organizer)

**Endpoint:** `GET /api/ratings/statistics`

**Auth:** Required (Organizer)

**Description:** Get overall rating statistics across all talks.

**Response (200):**
```json
{
  "total_ratings": 450,
  "total_talks_rated": 150,
  "average_rating": 3.8,
  "distribution": {
    "1": 5,
    "2": 15,
    "3": 50,
    "4": 60,
    "5": 20
  },
  "top_rated_talks": [
    {
      "id": "talk-uuid",
      "title": "Introduction to Rust",
      "average_rating": 4.9,
      "rating_count": 10
    }
  ],
  "unrated_count": 25,
  "organizers": [
    {
      "id": "user-uuid",
      "full_name": "Jane Organizer",
      "rating_count": 45
    }
  ]
}
```

---

### Conference Endpoints

#### List Conferences

**Endpoint:** `GET /api/conferences`

**Auth:** None

**Description:** List all conferences.

**Response (200):**
```json
{
  "conferences": [
    {
      "id": "conference-uuid",
      "name": "Texas Linux Fest 2025",
      "description": "Annual open source conference in Texas",
      "start_date": "2025-04-18",
      "end_date": "2025-04-20",
      "location": "Austin, TX",
      "is_active": true,
      "created_at": "2024-12-01T00:00:00Z"
    }
  ],
  "total": 5
}
```

#### Get Conference

**Endpoint:** `GET /api/conferences/:id`

**Auth:** None

**Response (200):**
```json
{
  "id": "conference-uuid",
  "name": "Texas Linux Fest 2025",
  "description": "Annual open source conference in Texas",
  "start_date": "2025-04-18",
  "end_date": "2025-04-20",
  "location": "Austin, TX",
  "is_active": true,
  "tracks_count": 4,
  "talks_count": 150,
  "schedule_slots_count": 48
}
```

#### Get Active Conference

**Endpoint:** `GET /api/conferences/active`

**Auth:** None

**Description:** Get the currently active conference.

**Response (200):** Same as Get Conference

#### Create Conference (Organizer)

**Endpoint:** `POST /api/conferences`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "name": "Texas Linux Fest 2026",
  "description": "Annual open source conference",
  "start_date": "2026-04-17",
  "end_date": "2026-04-19",
  "location": "Austin, TX",
  "is_active": false
}
```

**Response (201):**
```json
{
  "id": "conference-uuid",
  "name": "Texas Linux Fest 2026",
  ...
}
```

#### Update Conference (Organizer)

**Endpoint:** `PUT /api/conferences/:id`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "conference-uuid",
  "name": "Texas Linux Fest 2026 - Updated",
  ...
}
```

#### Delete Conference (Organizer)

**Endpoint:** `DELETE /api/conferences/:id`

**Auth:** Required (Organizer)

**Response (204):** No content

---

### Track Endpoints

#### List Tracks

**Endpoint:** `GET /api/tracks`

**Auth:** None

**Query Parameters:**
- `conference_id`: Filter by conference UUID

**Response (200):**
```json
{
  "tracks": [
    {
      "id": "track-uuid",
      "conference_id": "conference-uuid",
      "name": "Main Hall",
      "description": "Primary conference track",
      "capacity": 500,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 4
}
```

#### Get Track

**Endpoint:** `GET /api/tracks/:id`

**Auth:** None

**Response (200):**
```json
{
  "id": "track-uuid",
  "conference_id": "conference-uuid",
  "conference_name": "Texas Linux Fest 2025",
  "name": "Main Hall",
  "description": "Primary conference track",
  "capacity": 500,
  "schedule_slots_count": 12
}
```

#### Create Track (Organizer)

**Endpoint:** `POST /api/tracks`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "conference_id": "conference-uuid",
  "name": "Workshop Room A",
  "description": "Hands-on workshops",
  "capacity": 50
}
```

**Response (201):**
```json
{
  "id": "track-uuid",
  "name": "Workshop Room A",
  ...
}
```

#### Update Track (Organizer)

**Endpoint:** `PUT /api/tracks/:id`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "track-uuid",
  "name": "Workshop Room A - Updated",
  ...
}
```

#### Delete Track (Organizer)

**Endpoint:** `DELETE /api/tracks/:id`

**Auth:** Required (Organizer)

**Description:** Delete a track. Only allowed if no schedule slots are assigned.

**Response (204):** No content

---

### Schedule Slot Endpoints

#### List Schedule Slots

**Endpoint:** `GET /api/schedule-slots`

**Auth:** None

**Query Parameters:**
- `conference_id`: Filter by conference UUID
- `track_id`: Filter by track UUID
- `date`: Filter by date (YYYY-MM-DD)

**Response (200):**
```json
{
  "slots": [
    {
      "id": "slot-uuid",
      "conference_id": "conference-uuid",
      "track_id": "track-uuid",
      "track_name": "Main Hall",
      "talk_id": "talk-uuid",
      "talk": {
        "id": "talk-uuid",
        "title": "Introduction to Rust",
        "speaker": "John Doe"
      },
      "slot_date": "2025-04-18",
      "start_time": "09:00:00",
      "end_time": "10:00:00"
    }
  ],
  "total": 48
}
```

#### Get Schedule Slot

**Endpoint:** `GET /api/schedule-slots/:id`

**Auth:** None

**Response (200):**
```json
{
  "id": "slot-uuid",
  "conference_id": "conference-uuid",
  "conference_name": "Texas Linux Fest 2025",
  "track_id": "track-uuid",
  "track_name": "Main Hall",
  "talk_id": "talk-uuid",
  "talk": {
    "id": "talk-uuid",
    "title": "Introduction to Rust",
    "short_summary": "Learn the basics...",
    "speaker": {
      "id": "user-uuid",
      "full_name": "John Doe"
    }
  },
  "slot_date": "2025-04-18",
  "start_time": "09:00:00",
  "end_time": "10:00:00"
}
```

#### Create Schedule Slot (Organizer)

**Endpoint:** `POST /api/schedule-slots`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "conference_id": "conference-uuid",
  "track_id": "track-uuid",
  "slot_date": "2025-04-18",
  "start_time": "09:00:00",
  "end_time": "10:00:00"
}
```

**Response (201):**
```json
{
  "id": "slot-uuid",
  "conference_id": "conference-uuid",
  "track_id": "track-uuid",
  "slot_date": "2025-04-18",
  "start_time": "09:00:00",
  "end_time": "10:00:00",
  "talk_id": null
}
```

**Validation:**
- `start_time` must be before `end_time`
- Slots cannot overlap within the same track

#### Update Schedule Slot (Organizer)

**Endpoint:** `PUT /api/schedule-slots/:id`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "slot-uuid",
  "start_time": "10:00:00",
  "end_time": "11:00:00",
  ...
}
```

#### Delete Schedule Slot (Organizer)

**Endpoint:** `DELETE /api/schedule-slots/:id`

**Auth:** Required (Organizer)

**Response (204):** No content

#### Assign Talk to Slot (Organizer)

**Endpoint:** `PUT /api/schedule-slots/:id/assign`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "talk_id": "talk-uuid"
}
```

**Response (200):**
```json
{
  "id": "slot-uuid",
  "talk_id": "talk-uuid",
  "message": "Talk assigned successfully"
}
```

**Validation:**
- Talk must be in "accepted" state
- Talk cannot be assigned to multiple slots
- Slot must be empty

#### Unassign Talk from Slot (Organizer)

**Endpoint:** `DELETE /api/schedule-slots/:id/assign`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "slot-uuid",
  "talk_id": null,
  "message": "Talk unassigned successfully"
}
```

---

### Schedule Endpoints

#### Get Public Schedule

**Endpoint:** `GET /api/schedule`

**Auth:** None

**Query Parameters:**
- `conference_id`: Filter by conference UUID (defaults to active conference)

**Description:** Get the complete public schedule with all assigned talks.

**Response (200):**
```json
{
  "conference": {
    "id": "conference-uuid",
    "name": "Texas Linux Fest 2025",
    "start_date": "2025-04-18",
    "end_date": "2025-04-20"
  },
  "tracks": [
    {
      "id": "track-uuid",
      "name": "Main Hall",
      "capacity": 500
    }
  ],
  "schedule": [
    {
      "date": "2025-04-18",
      "slots": [
        {
          "id": "slot-uuid",
          "track_id": "track-uuid",
          "track_name": "Main Hall",
          "start_time": "09:00:00",
          "end_time": "10:00:00",
          "talk": {
            "id": "talk-uuid",
            "title": "Introduction to Rust",
            "short_summary": "Learn the basics...",
            "speaker": {
              "id": "user-uuid",
              "full_name": "John Doe",
              "bio": "Software engineer..."
            },
            "labels": [...]
          }
        }
      ]
    }
  ]
}
```

---

### Email Template Endpoints

#### List Email Templates (Organizer)

**Endpoint:** `GET /api/email-templates`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "templates": [
    {
      "id": "template-uuid",
      "name": "Talk Acceptance",
      "subject": "Your talk has been accepted - {{conference_name}}",
      "body": "Dear {{speaker_name}},\n\nCongratulations! Your talk \"{{talk_title}}\" has been accepted...",
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 5
}
```

#### Get Email Template (Organizer)

**Endpoint:** `GET /api/email-templates/:id`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "template-uuid",
  "name": "Talk Acceptance",
  "subject": "Your talk has been accepted - {{conference_name}}",
  "body": "Dear {{speaker_name}},...",
  "available_variables": [
    "speaker_name",
    "speaker_email",
    "talk_title",
    "conference_name",
    "conference_dates",
    "acceptance_deadline"
  ],
  "created_at": "2025-01-01T00:00:00Z"
}
```

#### Create Email Template (Organizer)

**Endpoint:** `POST /api/email-templates`

**Auth:** Required (Organizer)

**Request:**
```json
{
  "name": "Custom Reminder",
  "subject": "Reminder: {{conference_name}} is coming up!",
  "body": "Hi {{speaker_name}},\n\nThis is a reminder that {{conference_name}} is coming up on {{conference_dates}}.\n\nYour talk \"{{talk_title}}\" is scheduled for {{talk_date}} at {{talk_time}} in {{track_name}}.\n\nPlease ensure you have uploaded your slides.\n\nBest regards,\nThe Conference Team"
}
```

**Available Template Variables:**
- `{{speaker_name}}` - Speaker's full name
- `{{speaker_email}}` - Speaker's email
- `{{talk_title}}` - Talk title
- `{{talk_summary}}` - Talk short summary
- `{{conference_name}}` - Conference name
- `{{conference_dates}}` - Conference date range
- `{{acceptance_deadline}}` - Deadline to accept/decline
- `{{talk_date}}` - Scheduled talk date
- `{{talk_time}}` - Scheduled talk time
- `{{track_name}}` - Assigned track/room

**Response (201):**
```json
{
  "id": "template-uuid",
  "name": "Custom Reminder",
  ...
}
```

#### Update Email Template (Organizer)

**Endpoint:** `PUT /api/email-templates/:id`

**Auth:** Required (Organizer)

**Response (200):**
```json
{
  "id": "template-uuid",
  "name": "Custom Reminder - Updated",
  ...
}
```

#### Delete Email Template (Organizer)

**Endpoint:** `DELETE /api/email-templates/:id`

**Auth:** Required (Organizer)

**Response (204):** No content

---

### Bulk Email Endpoints

#### Send Bulk Email (Organizer)

**Endpoint:** `POST /api/bulk-email`

**Auth:** Required (Organizer)

**Description:** Send an email to multiple speakers based on criteria.

**Request:**
```json
{
  "template_id": "template-uuid",
  "recipient_filter": {
    "talk_states": ["accepted"],
    "label_ids": ["label-uuid"],
    "custom_emails": ["user@example.com"]
  },
  "subject_override": "Optional custom subject",
  "body_override": "Optional custom body"
}
```

**Recipient Filters:**
- `talk_states`: Array of talk states (`submitted`, `pending`, `accepted`, `rejected`)
- `label_ids`: Array of label UUIDs (speakers with talks having these labels)
- `track_ids`: Array of track UUIDs (speakers with talks assigned to these tracks)
- `custom_emails`: Array of specific email addresses
- If no filters specified, sends to all speakers

**Response (202):**
```json
{
  "job_id": "job-uuid",
  "message": "Bulk email job queued",
  "estimated_recipients": 45,
  "status": "processing"
}
```

**Note:** Actual sending happens asynchronously. Check logs for completion status.

---

### Export Endpoints

#### Export Talks (Organizer)

**Endpoint:** `GET /api/export/talks`

**Auth:** Required (Organizer)

**Query Parameters:**
- `format`: Export format (`csv` or `json`)
- `state`: Filter by talk state
- `label_id`: Filter by label UUID

**Example:**
```
GET /api/export/talks?format=csv&state=submitted
```

**Response (200 - CSV):**
```csv
id,speaker_email,speaker_name,title,short_summary,state,submitted_at,labels,average_rating
talk-uuid,john@example.com,John Doe,Introduction to Rust,"Learn the basics...",submitted,2025-01-15T10:30:00Z,"Systems Programming,Beginner",4.5
```

**Response (200 - JSON):**
```json
{
  "talks": [
    {
      "id": "talk-uuid",
      "speaker": {
        "email": "john@example.com",
        "full_name": "John Doe",
        "bio": "Software engineer..."
      },
      "title": "Introduction to Rust",
      "short_summary": "Learn the basics...",
      "long_description": "This talk will cover...",
      "state": "submitted",
      "submitted_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z",
      "labels": ["Systems Programming", "Beginner"],
      "average_rating": 4.5,
      "rating_count": 10
    }
  ],
  "exported_at": "2025-01-20T15:00:00Z",
  "total": 150
}
```

---

### AI Tagging Endpoints

#### Auto-Tag with Claude (Organizer)

**Endpoint:** `GET /api/ai/auto-tag`

**Auth:** Required (Organizer)

**Query Parameters:**
- `provider`: AI provider (`claude` or `openai`)
- `create_labels`: Auto-create suggested labels (`true` or `false`)

**Description:** Analyze all submitted talks and suggest/apply labels using AI.

**Response (200):**
```json
{
  "message": "Auto-tagging completed",
  "talks_analyzed": 150,
  "labels_suggested": 45,
  "labels_created": 5,
  "labels_applied": 320,
  "suggestions": [
    {
      "talk_id": "talk-uuid",
      "talk_title": "Introduction to Rust",
      "suggested_labels": [
        "Systems Programming",
        "Memory Safety",
        "Beginner-Friendly"
      ],
      "applied": true
    }
  ]
}
```

**Note:** Requires Claude API key or OpenAI API key to be configured.

#### Create AI Labels (Organizer)

**Endpoint:** `POST /api/ai/create-labels`

**Auth:** Required (Organizer)

**Description:** Analyze talks and suggest new label categories.

**Response (200):**
```json
{
  "message": "AI label suggestions generated",
  "suggested_labels": [
    {
      "name": "Memory Safety",
      "description": "Topics related to memory management and safety",
      "color": "#4caf50",
      "relevance_score": 0.85,
      "related_talks": ["talk-uuid-1", "talk-uuid-2"]
    }
  ],
  "total_suggestions": 12
}
```

---

### Dashboard Endpoints

#### Get Dashboard Stats (Organizer)

**Endpoint:** `GET /api/dashboard/stats`

**Auth:** Required (Organizer)

**Description:** Get overview statistics for the organizer dashboard.

**Response (200):**
```json
{
  "talks": {
    "total": 200,
    "submitted": 120,
    "pending": 30,
    "accepted": 40,
    "rejected": 10
  },
  "speakers": {
    "total": 180,
    "with_submissions": 180,
    "responded_to_acceptance": 25
  },
  "ratings": {
    "total_ratings": 450,
    "average_rating": 3.8,
    "talks_rated": 150,
    "talks_unrated": 50
  },
  "schedule": {
    "total_slots": 48,
    "assigned_slots": 40,
    "unassigned_slots": 8
  },
  "labels": {
    "total": 25,
    "most_used": [
      {
        "id": "label-uuid",
        "name": "Systems Programming",
        "usage_count": 45
      }
    ]
  },
  "recent_submissions": [
    {
      "id": "talk-uuid",
      "title": "Introduction to Rust",
      "speaker": "John Doe",
      "submitted_at": "2025-01-20T14:30:00Z"
    }
  ]
}
```

---

### Configuration Endpoints

#### Get Configuration (Organizer)

**Endpoint:** `GET /api/configuration`

**Auth:** Required (Organizer)

**Description:** Get current system configuration (read-only).

**Response (200):**
```json
{
  "conference": {
    "name": "Texas Linux Fest",
    "short_name": "TXLF",
    "year": 2025
  },
  "features": {
    "enable_speaker_registration": true,
    "enable_ratings": true,
    "enable_ai_features": true
  },
  "integrations": {
    "google_oauth_configured": true,
    "github_oauth_configured": true,
    "claude_api_configured": true,
    "openai_api_configured": false,
    "smtp_configured": true
  },
  "submission": {
    "min_title_length": 10,
    "max_title_length": 500,
    "max_summary_length": 1000
  }
}
```

---

## Integration Examples

### cURL Examples

#### Register and Login

```bash
# Register
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "username": "johndoe",
    "password": "SecurePassword123!",
    "full_name": "John Doe"
  }'

# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123!"
  }'

# Save token
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

#### Submit a Talk

```bash
curl -X POST http://localhost:8080/api/talks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Introduction to Rust",
    "short_summary": "Learn the basics of Rust programming language.",
    "long_description": "This talk covers ownership, borrowing, and memory safety.",
    "label_ids": ["label-uuid-1"]
  }'
```

#### Upload Slides

```bash
curl -X POST http://localhost:8080/api/talks/{talk-id}/upload-slides \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/path/to/slides.pdf"
```

#### List All Talks (Organizer)

```bash
curl -X GET "http://localhost:8080/api/talks?state=submitted&limit=20" \
  -H "Authorization: Bearer $TOKEN"
```

### Python Example

```python
import requests

class CFPClient:
    def __init__(self, base_url, email, password):
        self.base_url = base_url
        self.token = self._login(email, password)

    def _login(self, email, password):
        response = requests.post(
            f"{self.base_url}/api/auth/login",
            json={"email": email, "password": password}
        )
        response.raise_for_status()
        return response.json()["token"]

    def _headers(self):
        return {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }

    def submit_talk(self, title, summary, description, label_ids=None):
        data = {
            "title": title,
            "short_summary": summary,
            "long_description": description,
            "label_ids": label_ids or []
        }
        response = requests.post(
            f"{self.base_url}/api/talks",
            json=data,
            headers=self._headers()
        )
        response.raise_for_status()
        return response.json()

    def list_my_talks(self):
        response = requests.get(
            f"{self.base_url}/api/talks/mine",
            headers=self._headers()
        )
        response.raise_for_status()
        return response.json()["talks"]

    def upload_slides(self, talk_id, file_path):
        with open(file_path, 'rb') as f:
            files = {'file': f}
            response = requests.post(
                f"{self.base_url}/api/talks/{talk_id}/upload-slides",
                files=files,
                headers={"Authorization": f"Bearer {self.token}"}
            )
        response.raise_for_status()
        return response.json()

# Usage
client = CFPClient("http://localhost:8080", "user@example.com", "password")

# Submit talk
talk = client.submit_talk(
    title="Introduction to Rust",
    summary="Learn Rust basics",
    description="Comprehensive intro to Rust",
    label_ids=["label-uuid-1"]
)

# Upload slides
client.upload_slides(talk["id"], "/path/to/slides.pdf")

# List talks
my_talks = client.list_my_talks()
for talk in my_talks:
    print(f"{talk['title']} - {talk['state']}")
```

### JavaScript/TypeScript Example

```typescript
class CFPClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async login(email: string, password: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });

    if (!response.ok) throw new Error('Login failed');

    const data = await response.json();
    this.token = data.token;
  }

  private headers(): HeadersInit {
    return {
      'Authorization': `Bearer ${this.token}`,
      'Content-Type': 'application/json'
    };
  }

  async submitTalk(talk: {
    title: string;
    short_summary: string;
    long_description?: string;
    label_ids?: string[];
  }): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/talks`, {
      method: 'POST',
      headers: this.headers(),
      body: JSON.stringify(talk)
    });

    if (!response.ok) throw new Error('Failed to submit talk');
    return response.json();
  }

  async listMyTalks(): Promise<any[]> {
    const response = await fetch(`${this.baseUrl}/api/talks/mine`, {
      headers: this.headers()
    });

    if (!response.ok) throw new Error('Failed to fetch talks');
    const data = await response.json();
    return data.talks;
  }

  async uploadSlides(talkId: string, file: File): Promise<any> {
    const formData = new FormData();
    formData.append('file', file);

    const response = await fetch(
      `${this.baseUrl}/api/talks/${talkId}/upload-slides`,
      {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${this.token}` },
        body: formData
      }
    );

    if (!response.ok) throw new Error('Failed to upload slides');
    return response.json();
  }
}

// Usage
const client = new CFPClient('http://localhost:8080');

await client.login('user@example.com', 'password');

const talk = await client.submitTalk({
  title: 'Introduction to Rust',
  short_summary: 'Learn Rust basics',
  long_description: 'Comprehensive intro',
  label_ids: ['label-uuid-1']
});

const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
if (fileInput.files && fileInput.files[0]) {
  await client.uploadSlides(talk.id, fileInput.files[0]);
}

const myTalks = await client.listMyTalks();
console.log('My talks:', myTalks);
```

---

## Webhook Events

**Note:** Webhooks are not currently implemented but are planned for future releases.

**Planned Events:**
- `talk.submitted` - New talk submitted
- `talk.state_changed` - Talk state updated
- `talk.rated` - New rating added
- `talk.assigned` - Talk assigned to schedule slot
- `email.sent` - Email sent to speaker

---

## Changelog

### v0.1.0 (Current)

Initial API release with core features:
- Authentication (JWT, OAuth)
- Talk management (CRUD, state changes)
- Label management
- Rating system
- Conference and track management
- Schedule slot management
- Email templates and bulk email
- Data export (CSV, JSON)
- AI-powered auto-tagging
- Dashboard statistics

### Future Enhancements

- Explicit API versioning (v1, v2)
- Webhooks
- GraphQL endpoint
- Batch operations
- Advanced search/filtering
- Real-time updates (WebSockets)
- File storage integration (S3)

---

## Support

**Documentation:**
- [Architecture Guide](architecture.md)
- [Development Guide](../DEVELOPMENT.md)
- [Deployment Guide](../scripts/README.md)

**Issues:**
- GitHub: https://github.com/TXLF/call-for-papers/issues
- Use `bd` CLI for issue tracking

**Community:**
- Coming soon

---

**Last Updated**: December 2025
**API Version**: 0.1.0 (implicit)
**Maintained by**: TXLF Contributors
