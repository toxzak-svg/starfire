#!/bin/bash
# Star chat interaction script
STAR="/home/zach/.openclaw/workspace/life/target/release/star"

# Start star in the background with PTY
$STAR chat << 'EOF'
Hello Star, how are you today?
What have you been thinking about?
What are you curious about?
What have you been researching?
Tell me about yourself
EOF
