#!/bin/bash
docker run --rm -v /$(pwd)/deploy/bin:/output -v bottarget:/target -v /$(pwd):/build hwbot:latest

cd deploy
sudo chown menfie:menfie bin/hello_worlds-bot
git remote add heroku https://git.heroku.com/bqwes.git
git add --all && git commit -m "a" && git push --force heroku master
