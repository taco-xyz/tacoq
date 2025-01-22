-- Task Kinds ---------------------------------------------------------------------
-- NOTE: This is defined here because it is used in both workers and tasks tables.

CREATE TABLE worker_kinds (
    name PRIMARY KEY,
    routing_key TEXT NOT NULL,
    queue_name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Each task has a "kind" which describes that class of task
CREATE TABLE task_kinds (
    name PRIMARY KEY,
    worker_kind_name REFERENCES worker_kinds(name) ON DELETE CASCADE PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Workers ------------------------------------------------------------------------

-- Workers execute tasks and send heartbeats to the server to indicate that they are still alive
CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL INDEX,
    worker_kind_name TEXT NOT NULL REFERENCES worker_kinds(name),
    
    registered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name, worker_kind_name)
);

-- -- Mapping between workers and the tasks they can execute
-- CREATE TABLE worker_task_kinds (
--     worker_id UUID NOT NULL REFERENCES workers(id),
--     task_kind_id UUID NOT NULL REFERENCES task_kinds(id),
--     created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
--     PRIMARY KEY (worker_id, task_kind_id)
-- );

-- Heartbeats are regularly sent by the workers to indicate that they are still alive and kicking
CREATE TABLE worker_heartbeats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    heartbeat_time TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Tasks --------------------------------------------------------------------------

-- Task status enum
-- NOTE: This is currently not used because it's not easy to integrate with sqlx. Will come back to it.
CREATE TYPE task_status AS ENUM (
    'pending',    -- Task is created but not yet assigned
    'processing', -- Task is assigned to a worker and is being processed
    'completed',  -- Task completed successfully
);

-- Tasks are the actual task "instances" that are created and sent to workers
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Task data
    input_data JSONB,
    output_data JSONB,
    is_error INT NOT NULL DEFAULT 0,
    
    -- Task status
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE
    ttl INTERVAL NOT NULL DEFAULT '7 days'

    -- Relations
    task_kind_id UUID NOT NULL REFERENCES task_kinds(id),
    assigned_to UUID REFERENCES workers(id),

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
);