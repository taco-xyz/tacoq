from typing import Any, Optional
from uuid import UUID
from dataclasses import dataclass
from enum import Enum
from datetime import datetime


TaskInput = dict[str, Any]  # Maps to Option<serde_json::Value>
TaskOutput = dict[str, Any]  # Maps to Option<serde_json::Value>


class TaskStatus(str, Enum):
    """The status of a task.

    ### Possible Status:
    - `Pending`: Task is created but not yet assigned
    - `Queued`: Task has been assigned to a worker and sent to a queue
    - `Running`: Worker has started processing
    - `Completed`: Task completed successfully
    - `Failed`: Task failed to complete
    - `Cancelled`: Task was cancelled before completion
    - `Accepted`: Worker acknowledged receipt
    - `Paused`: Temporarily suspended
    - `Retrying`: Failed but attempting again
    - `Timeout`: Exceeded time limit
    - `Rejected`: Worker refused task
    - `Blocked`: Waiting on dependencies
    """

    PENDING = "pending"
    ACCEPTED = "accepted"
    QUEUED = "queued"
    RUNNING = "running"
    PAUSED = "paused"
    RETRYING = "retrying"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"
    TIMEOUT = "timeout"
    REJECTED = "rejected"
    BLOCKED = "blocked"


@dataclass
class TaskResult:
    """Task results contain the output or error data from a completed task.

    ### Parameters
    - `data`: The data of the task.
    - `is_error`: Whether the task failed.
    - `created_at`: The time the task was created.

    ### Methods
    - `from_dict`: Creates a TaskResult from a dictionary.
    """

    data: Optional[TaskOutput]
    is_error: bool
    created_at: datetime

    @staticmethod
    def from_dict(data: dict[str, Any]) -> "TaskResult":
        """Creates a TaskResult from a dictionary."""

        is_error: bool = data["output_data"] is None
        output_data = data["output_data"] if not is_error else data["error_data"]

        return TaskResult(
            data=output_data,
            is_error=is_error,
            created_at=datetime.fromisoformat(data["created_at"]),
        )


@dataclass
class TaskInstance:
    """Tasks are sent to workers to be executed with a specific payload.
    Workers are eligible for receiving certain tasks depending on their
    list of capabilities.

    ### Parameters
    - `id`: The ID of the task.
    - `task_kind`: The kind/class of the task.
    - `input_data`: The data of the task.
    - `status`: The status of the task.
    - `created_at`: The time the task was created.
    - `assigned_to`: The ID of the worker that is assigned to the task.
    - `result`: The result of the task.

    ### Properties
    - `has_finished`: Whether the task has finished.
    - `has_completed`: Whether the task has completed.
    - `has_failed`: Whether the task has failed.

    ### Methods
    - `from_dict`: Creates a TaskInstance from a dictionary.
    """

    id: UUID
    task_kind: str
    input_data: Optional[TaskInput]
    status: TaskStatus
    created_at: datetime
    assigned_to: Optional[UUID]
    result: Optional[TaskResult]

    @property
    def has_finished(self) -> bool:
        return self.status in [
            TaskStatus.COMPLETED,
            TaskStatus.FAILED,
            TaskStatus.CANCELLED,
            TaskStatus.TIMEOUT,
            TaskStatus.REJECTED,
        ]

    @property
    def has_completed(self) -> bool:
        return self.status == TaskStatus.COMPLETED

    @property
    def has_failed(self) -> bool:
        return self.status == TaskStatus.FAILED

    @staticmethod
    def from_dict(data: dict[str, Any]) -> "TaskInstance":
        """Creates a TaskInstance from a dictionary."""

        return TaskInstance(
            id=UUID(data["id"]),
            task_kind=data["task_kind"]["name"],
            input_data=data["input_data"],
            status=TaskStatus(data["status"].lower()),
            created_at=datetime.fromisoformat(data["created_at"]),
            assigned_to=UUID(data["assigned_to"]) if data["assigned_to"] else None,
            result=TaskResult.from_dict(data["result"]) if data["result"] else None,
        )
