#!/bin/bash

set -e
nohup /run/aionminer -l 127.0.0.1:8008 -u 0xa07e185919beef1e0a79fea78fcfabc24927c5067d758e514ad74b905a2bf137 -d 0 -t 1 &
echo "start aion_rust"

WS="${PWD}"
nohup ./target/release/aion --config=aion/cli/config_ci.toml &
sleep 7

cd ../aion_web3_test/
echo "===============start rpc bench==============="
node test_tools/benchtest_web3Requests.js --report "${WS}/test_results/report.html"
echo "===============start rpc test================"
yarn test --detectOpenHandles --runInBand

exit $?
