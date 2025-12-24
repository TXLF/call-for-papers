-- Conference/Event table
CREATE TABLE conferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    location VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tracks/Rooms table
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conference_id UUID NOT NULL REFERENCES conferences(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL, -- e.g., "Main Hall", "Track A", "Workshop Room"
    description TEXT,
    capacity INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tracks_conference_id ON tracks(conference_id);

-- Schedule slots table
CREATE TABLE schedule_slots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conference_id UUID NOT NULL REFERENCES conferences(id) ON DELETE CASCADE,
    track_id UUID NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    talk_id UUID REFERENCES talks(id) ON DELETE SET NULL,
    slot_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT check_time_order CHECK (start_time < end_time)
);

CREATE INDEX idx_schedule_slots_conference_id ON schedule_slots(conference_id);
CREATE INDEX idx_schedule_slots_track_id ON schedule_slots(track_id);
CREATE INDEX idx_schedule_slots_talk_id ON schedule_slots(talk_id);
CREATE INDEX idx_schedule_slots_date ON schedule_slots(slot_date, start_time);
