-- Email template types
CREATE TYPE email_template_type AS ENUM (
    'submission_confirmation',
    'talk_accepted',
    'talk_rejected',
    'talk_pending',
    'schedule_notification',
    'custom'
);

-- Email templates table
CREATE TABLE email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conference_id UUID REFERENCES conferences(id) ON DELETE CASCADE,
    template_type email_template_type NOT NULL,
    name VARCHAR(255) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL, -- Supports template variables like {{speaker_name}}, {{talk_title}}
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_templates_conference_id ON email_templates(conference_id);
CREATE INDEX idx_email_templates_type ON email_templates(template_type);

-- Email log (track sent emails)
CREATE TABLE email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    talk_id UUID REFERENCES talks(id) ON DELETE SET NULL,
    template_id UUID REFERENCES email_templates(id) ON DELETE SET NULL,
    recipient_email VARCHAR(255) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_by UUID REFERENCES users(id) ON DELETE SET NULL -- Organizer who sent it
);

CREATE INDEX idx_email_logs_user_id ON email_logs(user_id);
CREATE INDEX idx_email_logs_talk_id ON email_logs(talk_id);
CREATE INDEX idx_email_logs_sent_at ON email_logs(sent_at);
