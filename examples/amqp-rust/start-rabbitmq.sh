#!/usr/bin/env bash

HOSTNAME=spin-rabbitmq
USERNAME=user
PASSWORD=password
VERSION=3.8.22

if command -v podman &> /dev/null; then
  TOOL=podman
else
  TOOL=docker
fi

if [ "$($TOOL ps -a -q -f name=$HOSTNAME -f status=running)" ]; then
  echo "container running, removing before starting"
  $TOOL stop $HOSTNAME
fi

if [ "$($TOOL ps -a -q -f name=$HOSTNAME -f status=exited)" ]; then
  echo "container stopped, removing before starting"
  $TOOL rm $HOSTNAME
fi

$TOOL run -d \
  --hostname $HOSTNAME \
  --name $HOSTNAME \
  -p 5672:5672,15672:15672 \
  -e RABBITMQ_DEFAULT_USER=$USERNAME \
  -e RABBITMQ_DEFAULT_PASS=$PASSWORD \
  docker.io/library/rabbitmq:$VERSION-management

#TODO(justin) be less lazy about this
echo "waiting for container to accept imports"
sleep 10

echo "importing queue definitions"
curl -i -u $USERNAME:$PASSWORD \
  -X POST -H "content-type:application/json" -d @definitions.json \
  http://localhost:15672/api/definitions
