#!/usr/bin/env bash

HOSTNAME=spin-rabbitmq
USERNAME=user
PASSWORD=password
VERSION=3.8.22

if [ "$(docker ps -a -q -f name=$HOSTNAME -f status=running)" ]; then
  echo "container running, removing before starting"
  docker stop $HOSTNAME
fi

if [ "$(docker ps -a -q -f name=$HOSTNAME -f status=exited)" ]; then
  echo "container stopped, removing before starting"
  docker rm $HOSTNAME
fi

docker run -d \
  --hostname "$HOSTNAME" \
  --name "$HOSTNAME" \
  -p "5672:5672" \
  -p "15672:15672" \
  -e "RABBITMQ_DEFAULT_USER=$USERNAME" \
  -e "RABBITMQ_DEFAULT_PASS=$PASSWORD" \
  "rabbitmq:$VERSION-management"

# TODO(justin) be less lazy about this
echo "waiting for container to accept imports"
sleep 10

echo "importing queue definitions"
curl -i -u $USERNAME:$PASSWORD \
  -X POST -H "content-type:application/json" -d @definitions.json \
  http://localhost:15672/api/definitions
