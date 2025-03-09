-- Add migration script here
-- Add ttl_duration column to tasks table using an interval type
ALTER TABLE tasks
ADD COLUMN ttl_duration BIGINT;