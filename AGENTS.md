# Call for Papers - Project Overview

## Project Description

An open source call for papers (CFP) system designed for open source conferences, enabling speakers to submit talk proposals and organizers to manage the selection process. Built with Rust and Yew.rs, this self-hostable solution provides a complete workflow from submission to scheduling.

## Purpose

This project enables conference organizers to:
- Collect and manage talk submissions from speakers
- Rate and evaluate proposals collaboratively
- Select talks and build conference schedules
- Communicate with speakers throughout the process
- Export data for AI-powered analysis and tagging

## Tech Stack

- **Backend API**: Rust
- **Frontend**: Yew.rs (Rust WebAssembly framework)
- **Database**: PostgreSQL
- **Hosting**: Self-hosted on bare EC2 instances
- **License**: MIT

## Core Features

### Speaker Capabilities

**Talk Submission**
- Submit talk proposals with:
  - Title
  - Short summary
  - Optional long description
- Upload slides (can be added initially or returned to later)
- Select labels/tags for their talks
- View submission status (submitted, pending, accepted)
- Accept or decline talk acceptances

**Authentication**
- Custom username/password
- Social login options:
  - Apple
  - Google
  - Facebook
  - GitHub

### Organizer Capabilities

**Review and Rating**
- Read all submitted talks
- Rate submissions individually
- Use ratings to inform selection decisions
- Add or remove labels on any talk
- View aggregated ratings

**Talk Management**
- Export submissions for analysis via ChatGPT or Claude APIs
- Auto-tag submissions with AI-generated labels
- Manually add/remove labels
- Move talks through states:
  - Submitted
  - Pending (awaiting speaker confirmation)
  - Accepted
  - Rejected

**Schedule Building**
- Create conference schedules
- Define tracks (rooms)
- Assign time slots throughout conference days
- Organize accepted talks into the schedule

**Communication**
- Send emails to speakers
- Use customizable email templates
- Notify speakers of status changes

**Authentication**
- Same options as speakers (username/password or social logins)

### Data Model

**Talk States**
- **Submitted**: Initial state when speaker submits
- **Pending**: Talk accepted by organizers, awaiting speaker confirmation
- **Accepted**: Speaker has confirmed they will present
- **Rejected**: Not selected for the conference

**Talk Structure**
- Title (required)
- Short summary (required)
- Long description (optional)
- Slides (optional file upload)
- Labels/tags (auto-generated, organizer-added, or speaker-selected)
- Submission timestamp
- Current state
- Ratings from organizers

**Schedule Structure**
- Conference days
- Time slots
- Tracks/rooms
- Assigned talks

## Configuration

The system uses a configuration file to support different conferences and organizations:
- Default template provided for TXLF (Texas Linux Fest)
- Customizable for other conferences
- Email templates
- Branding and styling options

## Deployment

Designed for self-hosting:
- Deploy on bare EC2 instances
- Requires PostgreSQL database
- Static frontend assets served by Rust backend
- Single binary deployment

## AI Integration

Submissions can be exported and analyzed by:
- OpenAI ChatGPT API
- Anthropic Claude API

This enables automatic tagging and categorization of talks based on content analysis.

## Open Source

This project is open source under the MIT license, allowing anyone to:
- Use it for their own conferences
- Modify and customize it
- Contribute improvements back to the community
- Self-host without licensing fees
