#!/usr/bin/env bash

SESSION="risktopia"

# Check for existing session with same name
tmux has-session -t $SESSION 2>/dev/null
if [ $? = 0 ]; then
  echo "Session '$SESSION' already exists. Attaching..."
  tmux attach -t $SESSION
  exit 0
fi

# Create new detached session in frontend directory
tmux new-session -d -s $SESSION -c "$(pwd)/frontend" 'npm run dev'

# Split window and run backend
tmux split-window -h -c "$(pwd)/backend" 'cargo run'

# adjust layout
tmux select-layout even-horizontal

tmux attach -t $SESSION

