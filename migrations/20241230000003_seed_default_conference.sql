-- Seed a default conference for development and initial setup
-- This conference has the UUID that was hardcoded in frontend pages
INSERT INTO conferences (
    id,
    name,
    description,
    start_date,
    end_date,
    location,
    is_active
) VALUES (
    '00000000-0000-0000-0000-000000000000'::uuid,
    'Default Conference 2025',
    'This is the default conference created during initial setup. Update this conference or create a new one for your event.',
    '2025-01-01',
    '2025-12-31',
    'TBD',
    true
) ON CONFLICT (id) DO NOTHING;
