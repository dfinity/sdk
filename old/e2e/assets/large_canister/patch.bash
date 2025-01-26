#!/usr/bin/env bash
head -c $((1024 * 1024 * 8)) /dev/urandom >garbage.bin
