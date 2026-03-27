"""CLI for TTA-Crypto."""

import argparse
import asyncio
import sys
from datetime import datetime
from pathlib import Path

from tta_crypto.core import TemporalIndex, TemporalFact
from tta_crypto.ingest import ingest_crypto_data


async def cmd_ingest(args):
    """Ingest data from crypto APIs."""
    print("Ingesting crypto data...")
    
    facts = await ingest_crypto_data()
    print(f"Ingested {len(facts)} facts")
    
    # Save to index
    index = TemporalIndex()
    for fact in facts:
        index.add_fact(fact)
    
    index.save(args.output)
    print(f"Saved to {args.output}")


def cmd_query(args):
    """Query the temporal index."""
    index = TemporalIndex()
    index.load(args.index)
    
    # Parse timestamp
    if args.as_of:
        as_of = int(datetime.fromisoformat(args.as_of).timestamp())
    else:
        as_of = int(datetime.now().timestamp())
    
    facts = index.query(args.domain, as_of)
    
    print(f"Found {len(facts)} facts valid at {args.as_of or 'now'}:")
    for fact in facts[:10]:  # Show first 10
        print(f"  - {fact.content}")


def cmd_diff(args):
    """Generate temporal diff."""
    index = TemporalIndex()
    index.load(args.index)
    
    time1 = int(datetime.fromisoformat(args.time1).timestamp())
    time2 = int(datetime.fromisoformat(args.time2).timestamp())
    
    result = index.diff(args.domain, args.subject, time1, time2)
    
    print(f"\nTemporal Diff for {args.subject}")
    print(f"  At {result['time1']}: {result['at_time1']}")
    print(f"  At {result['time2']}: {result['at_time2']}")
    print(f"  Changed: {result['changed']}")


def cmd_history(args):
    """Get change history."""
    index = TemporalIndex()
    index.load(args.index)
    
    history = index.get_change_history(args.domain, args.subject)
    
    print(f"\nHistory for {args.subject}:")
    for fact in history:
        dt = datetime.fromtimestamp(fact.t_valid_from)
        print(f"  - {dt.date()}: {fact.content}")


def main():
    parser = argparse.ArgumentParser(description="TTA-Crypto CLI")
    subparsers = parser.add_subparsers(dest="command")
    
    # Ingest command
    ingest_parser = subparsers.add_parser("ingest", help="Ingest crypto data")
    ingest_parser.add_argument("-o", "--output", default="data/index.json", help="Output file")
    
    # Query command
    query_parser = subparsers.add_parser("query", help="Query temporal index")
    query_parser.add_argument("-i", "--index", default="data/index.json", help="Index file")
    query_parser.add_argument("-d", "--domain", required=True, help="Domain to query")
    query_parser.add_argument("--as-of", help="Timestamp (ISO format)")
    
    # Diff command
    diff_parser = subparsers.add_parser("diff", help="Generate temporal diff")
    diff_parser.add_argument("-i", "--index", default="data/index.json", help="Index file")
    diff_parser.add_argument("-d", "--domain", required=True, help="Domain")
    diff_parser.add_argument("-s", "--subject", required=True, help="Subject")
    diff_parser.add_argument("-t1", "--time1", required=True, help="First time (ISO)")
    diff_parser.add_argument("-t2", "--time2", required=True, help="Second time (ISO)")
    
    # History command
    history_parser = subparsers.add_parser("history", help="Get change history")
    history_parser.add_argument("-i", "--index", default="data/index.json", help="Index file")
    history_parser.add_argument("-d", "--domain", required=True, help="Domain")
    history_parser.add_argument("-s", "--subject", required=True, help="Subject")
    
    args = parser.parse_args()
    
    if args.command == "ingest":
        asyncio.run(cmd_ingest(args))
    elif args.command == "query":
        cmd_query(args)
    elif args.command == "diff":
        cmd_diff(args)
    elif args.command == "history":
        cmd_history(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
