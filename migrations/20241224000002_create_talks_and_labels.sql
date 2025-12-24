-- Talk state enum
CREATE TYPE talk_state AS ENUM ('submitted', 'pending', 'accepted', 'rejected');

-- Talks table
CREATE TABLE talks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    speaker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    short_summary TEXT NOT NULL,
    long_description TEXT,
    slides_url VARCHAR(1000), -- File path or URL to uploaded slides
    state talk_state NOT NULL DEFAULT 'submitted',
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_talks_speaker_id ON talks(speaker_id);
CREATE INDEX idx_talks_state ON talks(state);
CREATE INDEX idx_talks_submitted_at ON talks(submitted_at);

-- Labels/Tags table
CREATE TABLE labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    color VARCHAR(7), -- Hex color code for UI
    is_ai_generated BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_labels_name ON labels(name);

-- Talk labels junction table (many-to-many)
CREATE TABLE talk_labels (
    talk_id UUID NOT NULL REFERENCES talks(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    added_by UUID REFERENCES users(id) ON DELETE SET NULL, -- Who added this label
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (talk_id, label_id)
);

CREATE INDEX idx_talk_labels_talk_id ON talk_labels(talk_id);
CREATE INDEX idx_talk_labels_label_id ON talk_labels(label_id);

-- Ratings table (organizers rate talks)
CREATE TABLE ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    talk_id UUID NOT NULL REFERENCES talks(id) ON DELETE CASCADE,
    organizer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(talk_id, organizer_id) -- One rating per organizer per talk
);

CREATE INDEX idx_ratings_talk_id ON ratings(talk_id);
CREATE INDEX idx_ratings_organizer_id ON ratings(organizer_id);
