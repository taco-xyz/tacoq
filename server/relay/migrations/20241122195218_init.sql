-- Tasks are the actual task "instances" that are created and sent to workers
CREATE TABLE
    tasks (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        -- Task data
        task_kind_name TEXT,
        worker_kind_name TEXT,
        input_data BYTEA,
        output_data BYTEA,
        executed_by TEXT,
        is_error INT,
        priority INT,
        -- OpenTelemetry context carrier
        otel_ctx_carrier JSONB,
        -- Task status
        ttl_duration BIGINT,
        started_at TIMESTAMP,
        completed_at TIMESTAMP,
        created_at TIMESTAMP NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMP NOT NULL DEFAULT NOW ()
    );

CREATE INDEX tasks_ttl_idx ON tasks (
    (completed_at + interval '1 second' * ttl_duration)
);