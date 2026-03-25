"""Add conversation to dataset."""

import sys
import json
from dataset_builder import add_conversation, RAW_FILE, CLEAN_FILE, get_stats

if __name__ == "__main__":
    if len(sys.argv) >= 3:
        user_msg = sys.argv[1]
        assistant_msg = sys.argv[2]
        
        add_conversation(user_msg, assistant_msg)
        print("Done!")
    else:
        # Show stats
        stats = get_stats()
        print(f"Dataset stats:")
        print(f"  Raw conversations: {stats['raw_count']}")
        print(f"  Cleaned conversations: {stats['clean_count']}")
        print(f"\nFiles:")
        print(f"  Raw: {RAW_FILE}")
        print(f"  Cleaned: {CLEAN_FILE}")
