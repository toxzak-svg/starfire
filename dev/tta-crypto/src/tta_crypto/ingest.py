"""Data ingestion from crypto APIs."""

import httpx
import asyncio
from datetime import datetime
from typing import Dict, List, Optional
from .core import TemporalFact


class CryptoDataIngestor:
    """Ingest crypto data from various APIs."""
    
    def __init__(self):
        self.client = httpx.AsyncClient(timeout=30.0)
    
    async def fetch_defillama_tvl(self, timestamp: Optional[int] = None) -> List[TemporalFact]:
        """Fetch TVL data from DeFiLlama."""
        try:
            # Get current TVL
            response = await self.client.get("https://api.llama.fi/tvl")
            data = response.json()
            
            facts = []
            now = int(datetime.now().timestamp())
            
            # Convert to TemporalFacts
            for protocol, tvl in data.items():
                fact = TemporalFact(
                    fact_id=f"defillama_tvl_{protocol}_{now}",
                    content={
                        "metric": "TVL",
                        "protocol": protocol,
                        "value": tvl,
                        "unit": "USD"
                    },
                    t_valid_from=now,
                    source="DeFiLlama",
                    confidence=0.95,
                    domain="defi_tvl",
                    metadata={"category": "tvl"}
                )
                facts.append(fact)
            
            return facts
            
        except Exception as e:
            print(f"Error fetching DeFiLlama: {e}")
            return []
    
    async def fetch_prices(self, coins: List[str]) -> List[TemporalFact]:
        """Fetch prices from CoinGecko."""
        try:
            ids = ",".join(coins)
            response = await self.client.get(
                f"https://api.coingecko.com/api/v3/simple/price",
                params={
                    "ids": ids,
                    "vs_currencies": "usd",
                    "include_24hr_change": "true"
                }
            )
            data = response.json()
            
            facts = []
            now = int(datetime.now().timestamp())
            
            for coin, price_data in data.items():
                fact = TemporalFact(
                    fact_id=f"coingecko_price_{coin}_{now}",
                    content={
                        "metric": "price",
                        "asset": coin,
                        "value": price_data.get("usd", 0),
                        "change_24h": price_data.get("usd_24h_change", 0)
                    },
                    t_valid_from=now,
                    source="CoinGecko",
                    confidence=0.98,
                    domain="price",
                    metadata={"category": "price"}
                )
                facts.append(fact)
            
            return facts
            
        except Exception as e:
            print(f"Error fetching CoinGecko: {e}")
            return []
    
    async def fetch_whale_alerts(self, limit: int = 100) -> List[TemporalFact]:
        """Fetch recent whale alerts (mock for now)."""
        # WhaleAlert API requires subscription
        # This is a placeholder
        return []
    
    async def ingest_all(self) -> List[TemporalFact]:
        """Ingest all available data."""
        tasks = [
            self.fetch_defillama_tvl(),
            self.fetch_prices(["bitcoin", "ethereum", "solana", "cardano"])
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        facts = []
        for result in results:
            if isinstance(result, list):
                facts.extend(result)
        
        return facts
    
    async def close(self):
        await self.client.aclose()


# Convenience function
async def ingest_crypto_data() -> List[TemporalFact]:
    """Ingest crypto data."""
    ingestor = CryptoDataIngestor()
    try:
        return await ingestor.ingest_all()
    finally:
        await ingestor.close()


__all__ = ["CryptoDataIngestor", "ingest_crypto_data"]
