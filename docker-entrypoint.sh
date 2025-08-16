#!/bin/bash
# Start cron in the background
cron
# Follow the log file and output to stdout
tail -f /var/log/cron.log