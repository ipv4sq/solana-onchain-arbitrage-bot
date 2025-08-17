#!/bin/bash

# Find the most recent log file in the logs directory
LATEST_LOG=$(ls -t logs/bot_*.log 2>/dev/null | head -1)

if [ -z "$LATEST_LOG" ]; then
    echo "No log files found in logs/ directory"
    exit 1
fi

echo "Tailing $LATEST_LOG"
echo "Press Ctrl+C to stop"
echo "----------------------------------------"
tail -f "$LATEST_LOG"