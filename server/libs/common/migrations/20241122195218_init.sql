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
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        worker_kind_name TEXT NOT NULL REFERENCES worker_kinds (name),
        registered_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            UNIQUE (name, worker_kind_name)
    );

CREATE INDEX workers_name_idx ON workers (name);

-- Heartbeats are regularly sent by the workers to indicate that they are still alive and kicking
CREATE TABLE
    worker_heartbeats (
        worker_id UUID NOT NULL REFERENCES workers (id) ON DELETE CASCADE,
        heartbeat_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL,
            created_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            PRIMARY KEY (worker_id, heartbeat_time)
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
        input_data JSONB,
        output_data JSONB,
        is_error INT NOT NULL DEFAULT 0,
        -- Relations
        assigned_to UUID REFERENCES workers (id),
        worker_kind_name TEXT NOT NULL,
        -- Task status
        started_at TIMESTAMP
        WITH
            TIME ZONE,
            completed_at TIMESTAMP
        WITH
            TIME ZONE,
            ttl TIMESTAMP
        WITH
            TIME ZONE,
            -- Timestamps
            created_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            updated_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW ()
    );

CREATE INDEX tasks_ttl_idx ON tasks (ttl);