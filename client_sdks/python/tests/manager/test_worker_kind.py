import pytest
from aiohttp import ClientResponseError
from aioresponses import aioresponses

from manager.client import ManagerClient
from models.task import WorkerKindBrokerInfo


@pytest.mark.asyncio
async def test_worker_kind_registration_caching(mock_manager_client: ManagerClient):
    worker_kind = "test_kind"
    response_data = {
        "queue_name": f"worker_kind_{worker_kind}",
        "routing_key": f"worker_kind_{worker_kind}",
        "worker_kind": worker_kind,
    }

    with aioresponses() as m:
        # Should make one request and cache it
        m.put(  # type: ignore
            f"http://test/worker-kind/{worker_kind}",
            payload=response_data,
            status=200,
        )

        # This works because the request mock is only done once - the second request would crash
        # but it is never made because it hits the cache before trying to make the request.
        info1 = await mock_manager_client.get_worker_kind_broker_info(worker_kind)
        info2 = await mock_manager_client.get_worker_kind_broker_info(worker_kind)

        assert isinstance(info1, WorkerKindBrokerInfo)
        assert info1 == info2
        assert info1.queue_name == f"worker_kind_{worker_kind}"
        assert info1.routing_key == f"worker_kind_{worker_kind}"
        assert info1.worker_kind == worker_kind


@pytest.mark.asyncio
async def test_worker_kind_registration_different_kinds(
    mock_manager_client: ManagerClient,
):
    kinds = ["kind1", "kind2"]

    with aioresponses() as m:
        for kind in kinds:
            m.put(  # type: ignore
                f"http://test/worker-kind/{kind}",
                payload={
                    "queue_name": f"worker_kind_{kind}",
                    "routing_key": f"worker_kind_{kind}",
                    "worker_kind": kind,
                },
                status=200,
            )

        info1 = await mock_manager_client.get_worker_kind_broker_info(kinds[0])
        info2 = await mock_manager_client.get_worker_kind_broker_info(kinds[1])

        assert isinstance(info1, WorkerKindBrokerInfo)
        assert isinstance(info2, WorkerKindBrokerInfo)
        assert info1 != info2


@pytest.mark.asyncio
async def test_worker_kind_registration_error(mock_manager_client: ManagerClient):
    worker_kind = "test_kind"

    with aioresponses() as m:
        m.put(  # type: ignore
            f"http://test/worker-kind/{worker_kind}",
            status=500,
            body=b"Internal server error",
            repeat=True,
        )
        with pytest.raises(ClientResponseError) as exc_info:
            await mock_manager_client.get_worker_kind_broker_info(worker_kind)
        assert exc_info.value.status == 500
