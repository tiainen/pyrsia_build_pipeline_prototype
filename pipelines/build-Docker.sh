#!/usr/bin/bash

echo "#######################################################"
echo "#"
echo "# Starting ${1} build for:"
echo "#   ${2}"
echo "#"
echo "#######################################################"

fatal()
{
  echo "fatal: $1" 1>&2
  exit 1
}

BUILD_ID=${3}
IMAGE_NAME=${4}
IMAGE_REFERENCE=${5}

# create final artifacts dir
mkdir -p artifacts || fatal "Could not create directory: \"artifacts\""

# create and enter docker dir
mkdir -p docker || fatal "Could not create directory: \"docker\""
cd docker

# get docker hub token
DOCKER_IO_TOKEN=$(curl "https://auth.docker.io/token?client_id=Pyrsia&service=registry.docker.io&scope=repository:${IMAGE_NAME}:pull" | jq -r .token) || fatal "Failed to fetch authorization token for docker.io"

# download manifest
# todo: if we ever support tags, we should try to download the manifest list first:
# curl "https://registry-1.docker.io/v2/${IMAGE_NAME}/manifests/${IMAGE_REFERENCE}" \
#  -H "Authorization: Bearer ${DOCKER_IO_TOKEN}" \
#  -H "Accept: application/vnd.docker.distribution.manifest.list.v2+json" \
#  -o "manifest.list"
# if it returns a Content-Type header with the same value as the Accept header, we have a list,
# otherwise just download the regular v2 manifest.

curl -L "https://registry-1.docker.io/v2/${IMAGE_NAME}/manifests/${IMAGE_REFERENCE}" \
 -H "Authorization: Bearer ${DOCKER_IO_TOKEN}" \
 -H "Accept: application/vnd.docker.distribution.manifest.v2+json" \
 -o "manifest"

# download config
CONFIG_DIGEST=$(cat manifest | jq -r .config.digest)
curl -L "https://registry-1.docker.io/v2/${IMAGE_NAME}/blobs/${CONFIG_DIGEST}" \
 -H "Authorization: Bearer ${DOCKER_IO_TOKEN}" \
 -o "${CONFIG_DIGEST}.blob"

# download blobs
for b in "$(cat manifest | jq -r .layers[].digest)"; do
  BLOB_DIGEST=${b}
  curl -L "https://registry-1.docker.io/v2/${IMAGE_NAME}/blobs/${BLOB_DIGEST}" \
   -H "Authorization: Bearer ${DOCKER_IO_TOKEN}" \
   -o "${BLOB_DIGEST}.blob"
done

# copy artifacts
mv manifest *.blob ../artifacts/ || fatal "Failed to copy artifacts"
