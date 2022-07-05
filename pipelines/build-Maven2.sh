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
GIT_REPO=${4}
GIT_TAG=${5}
BUILD_SPEC_URL=${6}

# create final artifacts dir
mkdir artifacts || fatal "Could not create directory: \"artifacts\""

# clone repository
git clone ${GIT_REPO} repo || fatal "Failed to clone git repository"
cd repo

# checkout out specific tag
git checkout -f `git rev-parse ${GIT_TAG}^{commit}` || fatal "Failed to checkout git tag"

# manage buildspec
BUILD_SPEC=.pyrsia.buildspec
if [ ! -f "${BUILD_SPEC}" ]; then
    curl $BUILD_SPEC_URL > ${BUILD_SPEC} || fatal "Failed to fetch buildspec"
fi
mv ${BUILD_SPEC} ../
. $(pwd)/../${BUILD_SPEC}

# start build
export JAVA_HOME=/opt/java11
mvn clean package || fatal "Failed to execute maven build"

# copy artifacts
mkdir ../artifacts/0
cp ${artifact} ../artifacts/0/artifact.file || fatal "Failed to copy artifact: ${artifact}"
cp ${sourceArtifact} ../artifacts/0/source_artifact.file || fatal "Failed to copy source artifact: ${sourceArtifact}"
