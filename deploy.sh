#!/bin/bash
docker build -t hwbot .
docker run --rm -v /$(pwd)/deploy/bin:/output hwbot:latest

cd deploy
sudo chown menfie:menfie bin/hello_worlds-bot
git remote add heroku https://git.heroku.com/bqwes.git
git add --all && git commit -m "a" && git push --force heroku master
