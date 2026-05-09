#!/bin/bash
set -e

# rockboxd creates the FIFO itself via pcm_fifo_set_path(), but it must start
# before snapserver — if snapserver opens the pipe first it sees immediate EOF.

rockboxd &
ROCKBOX_PID=$!

# Wait until rockboxd has created and opened the FIFO (holds a writer fd so
# snapserver never sees EOF between tracks).
i=0
while [ ! -p /tmp/rockbox.fifo ]; do
  i=$((i + 1))
  if [ $i -ge 30 ]; then
    echo "error: /tmp/rockbox.fifo not created after 30 s — rockboxd may have failed" >&2
    exit 1
  fi
  sleep 1
done

snapserver &
SNAP_PID=$!

# Exit when either process dies so Docker can restart the container.
wait -n $ROCKBOX_PID $SNAP_PID
