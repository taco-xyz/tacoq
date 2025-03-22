-- Description: Initial migration for the task queue database
-- Worker Kinds ---------------------------------------------------------------------
-- Worker kinds are the types of workers that can be registered with the server
CREATE TABLE
    worker_kinds (
        name TEXT NOT NULL PRIMARY KEY,
        routing_key TEXT NOT NULL,
        queue_name TEXT NOT NULL,
        created_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW ()
    );

-- Workers ------------------------------------------------------------------------
-- Workers execute tasks and send heartbeats to the server to indicate that they are still alive
CREATE TABLE
    workers (
        name TEXT PRIMARY KEY,
        worker_kind_name TEXT NOT NULL REFERENCES worker_kinds (name),
        registered_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW ()
    );

-- Heartbeats are regularly sent by the workers to indicate that they are still alive and kicking
CREATE TABLE
    worker_heartbeats (
        worker_name TEXT NOT NULL REFERENCES workers (name) ON DELETE CASCADE,
        heartbeat_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL,
            created_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            PRIMARY KEY (worker_name, heartbeat_time)
    );

-- Tasks --------------------------------------------------------------------------
-- Task status enum
-- NOTE: This is currently not used because it's not easy to integrate with sqlx. Will come back to it.
CREATE TYPE task_status AS ENUM (
    'pending', -- Task is created but not yet assigned
    'processing', -- Task is assigned to a worker and is being processed
    'completed' -- Task completed successfully
);

-- Tasks are the actual task "instances" that are created and sent to workers
CREATE TABLE
    tasks (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        -- Task data
        task_kind_name TEXT NOT NULL,
        input_data BYTEA,
        output_data BYTEA,
        is_error INT NOT NULL DEFAULT 0,
        status VARCHAR(255) NOT NULL,
        priority INT NOT NULL DEFAULT 0,
        -- OpenTelemetry context carrier
        otel_ctx_carrier JSONB,
        -- Relations
        executed_by TEXT REFERENCES workers (name),
        worker_kind_name TEXT NOT NULL REFERENCES worker_kinds (name),
        -- Task status
        ttl_duration BIGINT NOT NULL,
        started_at TIMESTAMP,
        completed_at TIMESTAMP,
        created_at TIMESTAMP NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMP NOT NULL DEFAULT NOW ()
    );

CREATE INDEX tasks_ttl_idx ON tasks (
    (completed_at + interval '1 second' * ttl_duration)
);