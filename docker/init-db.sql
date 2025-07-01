-- Create events table
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    key_code VARCHAR(20),
    masked_text TEXT,
    window_title TEXT,
    application VARCHAR(255),
    metadata JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_events_timestamp ON events(timestamp DESC);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_application ON events(application);
CREATE INDEX idx_events_metadata ON events USING GIN (metadata);

-- Create full-text search configuration
CREATE TEXT SEARCH CONFIGURATION keyai_search (COPY = pg_catalog.english);

-- Add full-text search column
ALTER TABLE events ADD COLUMN search_vector tsvector;

-- Create trigger to update search vector
CREATE OR REPLACE FUNCTION update_search_vector() RETURNS trigger AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('keyai_search', COALESCE(NEW.masked_text, '')), 'A') ||
        setweight(to_tsvector('keyai_search', COALESCE(NEW.window_title, '')), 'B') ||
        setweight(to_tsvector('keyai_search', COALESCE(NEW.application, '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_events_search_vector
    BEFORE INSERT OR UPDATE ON events
    FOR EACH ROW
    EXECUTE FUNCTION update_search_vector();

-- Create index on search vector
CREATE INDEX idx_events_search_vector ON events USING GIN (search_vector);

-- Create table for semantic embeddings (future use)
CREATE TABLE IF NOT EXISTS embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    vector REAL[],
    model_version VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_embeddings_event_id ON embeddings(event_id);

-- Create statistics table
CREATE TABLE IF NOT EXISTS statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    total_events BIGINT DEFAULT 0,
    events_by_type JSONB,
    events_by_app JSONB,
    events_by_hour JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_statistics_date ON statistics(date);

-- Create sessions table for tracking user sessions
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    events_count BIGINT DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_started_at ON sessions(started_at DESC);

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO keyai;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO keyai; 