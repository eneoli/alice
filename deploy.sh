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
echo "$SSH_KNOWN_HOSTS" > ~/.ssh/known_hosts

rm -rf dist.zip
zip -r dist.zip dist/

ssh -i ~/.ssh/deploy.key $DEPLOYMENT_USER@$DEPLOYMENT_SERVER "rm -rf $DEPLOYMENT_RES_PATH/*"
scp -i ~/.ssh/deploy.key dist.zip $DEPLOYMENT_USER@$DEPLOYMENT_SERVER:$DEPLOYMENT_RES_PATH
ssh -i ~/.ssh/deploy.key $DEPLOYMENT_USER@$DEPLOYMENT_SERVER "unzip $DEPLOYMENT_RES_PATH/dist.zip && mv dist/* ../ && rm -rf dist/ && rm -rf dist.zip"

echo -e "\033[1;32m"
echo "Done ^-^"
echo -e "\033[0m"