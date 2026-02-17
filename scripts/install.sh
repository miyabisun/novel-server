#!/bin/sh
set -e
cd "$(dirname "$0")/.."

pwd
npm install
echo "-----"
cd client
pwd
npm install
echo "-----"
cd ..

if [ ! -f .env ]; then
  npm run init
else
  echo ".env already exists, skipping."
fi
echo "-----"

pwd
npx prisma db push
