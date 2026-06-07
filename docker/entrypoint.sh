#!/bin/bash
set -e

# Default audio output is CMAF (HLS + DASH) — the web UI's <audio> tag attaches
# to the HLS stream automatically, so no external client is needed.
#
# Snapcast (snapserver) is still started in the background so users who switch
# `audio_output = "fifo"` in settings.toml can immediately stream to snapclient
# without rebuilding the image. When the active sink is CMAF, snapserver runs
# idle and consumes effectively no resources.

rockboxd &
ROCKBOX_PID=$!

snapserver &
SNAP_PID=$!

# Exit when either process dies so Docker can restart the container.
wait -n $ROCKBOX_PID $SNAP_PID
