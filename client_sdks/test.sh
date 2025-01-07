#!/bin/bash
set -e

# Build the application container
echo "Building application container..."
docker build -t tacoq-manager:latest -f ../server/services/manager/Dockerfile ..

# Start the test environment
echo "Starting test environment..."
docker compose -f docker-compose.test.yml up -d

# Wait a moment for containers to start
sleep 5

# Show logs from all containers
echo "Container logs:"
docker compose -f docker-compose.test.yml logs

# Show detailed container info
echo "Container info:"
docker compose -f docker-compose.test.yml ps -a

# Try running the container directly to see the error
echo "Testing container directly:"
docker run --rm \
  --network client_sdks_default \
  -e DATABASE_URL=postgres://user:password@postgres:5432/tacoq \
  -e AMQP_URL=amqp://user:password@rabbitmq:5672/ \
  -e RUST_LOG=debug \
  tacoq-manager:latest

# Show container status
echo "Container status:"
docker compose -f docker-compose.test.yml ps

# Show logs if any container is not healthy
if docker compose -f docker-compose.test.yml ps | grep -v "healthy"; then
    echo "Some containers are not healthy. Showing logs:"
    docker compose -f docker-compose.test.yml logs
fi

echo "Test environment is ready at http://localhost:3000"
echo "To stop the environment, run: docker compose -f docker-compose.test.yml down"
