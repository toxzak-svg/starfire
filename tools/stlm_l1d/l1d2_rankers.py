from l1d2_core import *


class PairwiseLinearRanker:
    def __init__(self, dimension: int, learning_rate: float, l2: float, epochs: int) -> None:
        self.weights = [0.0] * dimension
        self.learning_rate = learning_rate
        self.l2 = l2
        self.epochs = epochs

    def score(self, features: Mapping[int, float]) -> float:
        return sum(self.weights[index] * value for index, value in features.items())

    def train(
        self,
        tournaments: Sequence[Tournament],
        feature_fn: Callable[[str, Sequence[int]], dict[int, float]],
        seed: int,
    ) -> None:
        pairs: list[tuple[dict[int, float], dict[int, float]]] = []
        for tournament in tournaments:
            gold = candidate_by_id(tournament, tournament.gold_candidate_id)
            gold_features = feature_fn(gold.text, tournament.context)
            for candidate in tournament.candidates:
                if candidate.candidate_id == gold.candidate_id:
                    continue
                pairs.append((gold_features, feature_fn(candidate.text, tournament.context)))
        if not pairs:
            return
        for epoch in range(self.epochs):
            order = list(range(len(pairs)))
            random.Random(seed + epoch * 104729).shuffle(order)
            step = self.learning_rate / math.sqrt(epoch + 1.0)
            for pair_index in order:
                positive, negative = pairs[pair_index]
                margin = self.score(positive) - self.score(negative)
                clipped = max(-30.0, min(30.0, margin))
                gradient_scale = 1.0 / (1.0 + math.exp(clipped))
                touched = set(positive) | set(negative)
                for index in touched:
                    difference = positive.get(index, 0.0) - negative.get(index, 0.0)
                    self.weights[index] += step * (
                        gradient_scale * difference - self.l2 * self.weights[index]
                    )


def candidate_by_id(tournament: Tournament, candidate_id: int) -> Candidate:
    for candidate in tournament.candidates:
        if candidate.candidate_id == candidate_id:
            return candidate
    raise KeyError(candidate_id)


def stratified_group_split(
    tournaments: Sequence[Tournament], seed: int
) -> dict[str, list[Tournament]]:
    groups_by_category: dict[str, list[str]] = defaultdict(list)
    group_category: dict[str, str] = {}
    for tournament in tournaments:
        previous = group_category.setdefault(tournament.group_id, tournament.category)
        if previous != tournament.category:
            raise ValueError(f"group {tournament.group_id!r} crosses categories")
    for group_id, category in group_category.items():
        groups_by_category[category].append(group_id)
    split_groups = {"train": set(), "dev": set(), "test": set()}
    for category in CATEGORY_ORDER:
        groups = sorted(groups_by_category[category])
        if len(groups) < 6:
            raise ValueError(f"category {category!r} needs at least six groups")
        random.Random(seed ^ stable_hash(category, 2**31 - 1)).shuffle(groups)
        test_groups = groups[:2]
        dev_group = groups[2]
        train_groups = groups[3:]
        split_groups["test"].update(test_groups)
        split_groups["dev"].add(dev_group)
        split_groups["train"].update(train_groups)
    splits = {
        name: [tournament for tournament in tournaments if tournament.group_id in groups]
        for name, groups in split_groups.items()
    }
    for name, records in splits.items():
        categories = {record.category for record in records}
        if categories != set(CATEGORY_ORDER):
            raise ValueError(f"{name} split lacks category coverage")
    if (
        split_groups["train"] & split_groups["dev"]
        or split_groups["train"] & split_groups["test"]
        or split_groups["dev"] & split_groups["test"]
    ):
        raise ValueError("group leakage across splits")
    return splits


def transform_punctuation(text: str) -> str:
    translated = "".join(
        " " if unicodedata.category(character).startswith("P") else character
        for character in text
    )
    return re.sub(r"\s+", " ", translated).strip()


def transform_whitespace(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def transform_unicode(text: str) -> str:
    normalized = unicodedata.normalize("NFKC", text)
    replacements = {
        "\u2018": "'",
        "\u2019": "'",
        "\u201c": '"',
        "\u201d": '"',
        "\u2013": "-",
        "\u2014": "-",
        "\u2026": "...",
        "\u00a0": " ",
    }
    return "".join(replacements.get(character, character) for character in normalized)


def transformed_tournament(
    tournament: Tournament, transform: Callable[[str], str]
) -> Tournament:
    return Tournament(
        tournament_id=tournament.tournament_id,
        group_id=tournament.group_id,
        category=tournament.category,
        context=tournament.context,
        gold_candidate_id=tournament.gold_candidate_id,
        semantic_signature=tournament.semantic_signature,
        candidates=tuple(
            Candidate(
                candidate_id=candidate.candidate_id,
                text=transform(candidate.text),
                rule_score=candidate.rule_score,
                semantic_valid=candidate.semantic_valid,
                slots_preserved=candidate.slots_preserved,
                identity_conflicts=candidate.identity_conflicts,
            )
            for candidate in tournament.candidates
        ),
    )


def shuffled_contexts(
    tournaments: Sequence[Tournament], seed: int
) -> list[Tournament]:
    by_category: dict[str, list[Tournament]] = defaultdict(list)
    for tournament in tournaments:
        by_category[tournament.category].append(tournament)
    result: list[Tournament] = []
    for category in CATEGORY_ORDER:
        records = sorted(by_category[category], key=lambda item: item.tournament_id)
        contexts = [record.context for record in records]
        random.Random(seed ^ stable_hash(f"context:{category}", 2**31 - 1)).shuffle(
            contexts
        )
        if len(contexts) > 1 and all(
            left == right
            for left, right in zip(contexts, (record.context for record in records))
        ):
            contexts = contexts[1:] + contexts[:1]
        for record, context in zip(records, contexts):
            result.append(
                Tournament(
                    tournament_id=record.tournament_id,
                    group_id=record.group_id,
                    category=record.category,
                    context=context,
                    gold_candidate_id=record.gold_candidate_id,
                    semantic_signature=record.semantic_signature,
                    candidates=record.candidates,
                )
            )
    return sorted(result, key=lambda item: item.tournament_id)


def shuffled_labels(
    tournaments: Sequence[Tournament], seed: int
) -> list[Tournament]:
    result = []
    for tournament in tournaments:
        candidates = list(tournament.candidates)
        rng = random.Random(seed ^ stable_hash(tournament.tournament_id, 2**31 - 1))
        alternatives = [
            candidate.candidate_id
            for candidate in candidates
            if candidate.candidate_id != tournament.gold_candidate_id
        ]
        shuffled_gold = alternatives[rng.randrange(len(alternatives))]
        result.append(
            Tournament(
                tournament_id=tournament.tournament_id,
                group_id=tournament.group_id,
                category=tournament.category,
                context=tournament.context,
                gold_candidate_id=shuffled_gold,
                semantic_signature=tournament.semantic_signature,
                candidates=tournament.candidates,
            )
        )
    return result


def score_map(
    tournament: Tournament,
    scorer: Callable[[Tournament, Candidate], float],
) -> dict[int, float]:
    return {
        candidate.candidate_id: scorer(tournament, candidate)
        for candidate in tournament.candidates
    }


def rank_candidates(scores: Mapping[int, float]) -> list[int]:
    return [
        candidate_id
        for candidate_id, _ in sorted(
            scores.items(), key=lambda item: (-item[1], item[0])
        )
    ]


def evaluate_ranker(
    tournaments: Sequence[Tournament],
    scorer: Callable[[Tournament, Candidate], float],
) -> dict[str, Any]:
    if not tournaments:
        return {
            "tournaments": 0,
            "top1_accuracy_bps": 0,
            "mean_reciprocal_rank_bps": 0,
            "pairwise_accuracy_bps": 0,
            "by_category": {},
            "selections": [],
        }
    top1 = 0
    reciprocal_ranks = []
    pairwise = []
    by_category_values: dict[str, list[int]] = defaultdict(list)
    selections = []
    for tournament in sorted(tournaments, key=lambda item: item.tournament_id):
        scores = score_map(tournament, scorer)
        ranking = rank_candidates(scores)
        gold_index = ranking.index(tournament.gold_candidate_id)
        won = int(gold_index == 0)
        top1 += won
        reciprocal_ranks.append(1.0 / (gold_index + 1))
        gold_score = scores[tournament.gold_candidate_id]
        for candidate_id, value in scores.items():
            if candidate_id != tournament.gold_candidate_id:
                pairwise.append(
                    1.0 if gold_score > value else 0.5 if gold_score == value else 0.0
                )
        by_category_values[tournament.category].append(won)
        selections.append(
            {
                "tournament_id": tournament.tournament_id,
                "gold_candidate_id": tournament.gold_candidate_id,
                "selected_candidate_id": ranking[0],
                "gold_rank": gold_index + 1,
                "scores": {
                    str(key): round(value, 6)
                    for key, value in sorted(scores.items())
                },
            }
        )
    return {
        "tournaments": len(tournaments),
        "top1_accuracy_bps": round(top1 / len(tournaments) * 10_000),
        "mean_reciprocal_rank_bps": round(
            statistics.fmean(reciprocal_ranks) * 10_000
        ),
        "pairwise_accuracy_bps": round(statistics.fmean(pairwise) * 10_000),
        "by_category": {
            category: (
                round(statistics.fmean(by_category_values[category]) * 10_000)
                if by_category_values[category]
                else None
            )
            for category in CATEGORY_ORDER
        },
        "selections": selections,
    }


def selection_stability(
    original: Mapping[str, Any], controlled: Mapping[str, Any]
) -> int:
    left = {
        item["tournament_id"]: item["selected_candidate_id"]
        for item in original["selections"]
    }
    right = {
        item["tournament_id"]: item["selected_candidate_id"]
        for item in controlled["selections"]
    }
    shared = sorted(set(left) & set(right))
    if not shared:
        return 0
    return round(
        sum(left[item] == right[item] for item in shared) / len(shared) * 10_000
    )


def choose_linear_ranker(
    train: Sequence[Tournament],
    dev: Sequence[Tournament],
    feature_fn: Callable[[str, Sequence[int]], dict[int, float]],
    dimension: int,
    seed: int,
) -> tuple[PairwiseLinearRanker, dict[str, Any]]:
    configurations = (
        (0.08, 0.00001, 24),
        (0.05, 0.0001, 32),
        (0.03, 0.001, 40),
    )
    best: tuple[int, int, PairwiseLinearRanker, tuple[float, float, int]] | None = (
        None
    )
    for index, (learning_rate, l2, epochs) in enumerate(configurations):
        ranker = PairwiseLinearRanker(dimension, learning_rate, l2, epochs)
        ranker.train(train, feature_fn, seed + index * 1009)
        metrics = evaluate_ranker(
            dev,
            lambda tournament, candidate: ranker.score(
                feature_fn(candidate.text, tournament.context)
            ),
        )
        key = (metrics["top1_accuracy_bps"], metrics["mean_reciprocal_rank_bps"])
        if best is None or key > best[:2]:
            best = (key[0], key[1], ranker, (learning_rate, l2, epochs))
    assert best is not None
    return best[2], {
        "learning_rate": best[3][0],
        "l2": best[3][1],
        "epochs": best[3][2],
        "dev_top1_accuracy_bps": best[0],
        "dev_mean_reciprocal_rank_bps": best[1],
    }
