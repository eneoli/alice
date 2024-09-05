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

cd frontend
npm install
npm run build:$BUILD_MODE
cd ..

echo ""
echo "--> Uploading to deployment server..."
echo ""

mkdir -p ~/.ssh/
echo "$SSH_KEY" > ~/.ssh/deploy.key
sudo chmod 600 ~/.ssh/deploy.key
echo "$SSH_KNOWN_HOSTS" > ~/.ssh/known_host

ssh $DEPLOYMENT_USER@$DEPLOYMENT_SERVER "rm -rf $DEPLOYMENT_RES_PATH/*"
scp -r dist $DEPLOYMENT_USER@$DEPLOYMENT_SERVER:$DEPLOYMENT_RES_PATH

echo -e "\033[1;32m"
echo "Done ^-^"
echo -e "\033[0m"