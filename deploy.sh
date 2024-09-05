#!/usr/bin/env bash

set -e

SCRIPT_DIR=$(pwd)
ROOT_DIR=$SCRIPT_DIR
BUILD_DIR=$SCRIPT_DIR/dist

echo "==== Alice Deployment ===="
echo ""
echo "--> Compiling WASM binary..."
echo ""

wasm-pack build

echo ""
echo "--> Compiling frontend..."
echo ""

npm run build:$BUILD_MODE

echo ""
echo "--> Uploading to deployment server..."
echo ""

ssh-add - <<< "${SSH_KEY}"
ssh $DEPLOYMENT_USER@$DEPLOYMENT_SERVER "rm -rf $DEPLOYMENT_RES_PATH/*"
scp -r dist $DEPLOYMENT_USER@$DEPLOYMENT_SERVER:$DEPLOYMENT_RES_PATH

echo -e "\033[1;32m"
echo "Done ^-^"
echo -e "\033[0m"