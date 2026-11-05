import time
from typing import Any

import numpy as np


class VectorStoreEngine:
    """Small in-memory cosine index for caller-supplied embeddings.

    This class does not generate embeddings or claim semantic understanding; a
    model integration can supply vectors through ``add_document``.
    """

    def __init__(self, embedding_dimension: int = 384) -> None:
        if embedding_dimension <= 0:
            raise ValueError("embedding_dimension must be positive")
        self.dimension = embedding_dimension
        self.vectors: list[np.ndarray] = []
        self.metadata: list[dict[str, Any]] = []
        self._matrix_cache: np.ndarray | None = None

    def _normalize(self, values: list[float] | np.ndarray) -> np.ndarray:
        vector = np.asarray(values, dtype=np.float32)
        if vector.ndim != 1 or vector.shape[0] != self.dimension:
            raise ValueError(f"expected a 1D vector of length {self.dimension}")
        if not np.all(np.isfinite(vector)):
            raise ValueError("vector values must be finite")
        norm = float(np.linalg.norm(vector))
        if norm == 0.0:
            raise ValueError("zero vectors cannot be indexed or searched")
        return vector / norm

    def add_document(self, text: str, source: str, raw_vector: list[float]) -> None:
        vector = self._normalize(raw_vector)
        self.vectors.append(vector)
        self.metadata.append({"text": text, "source": source, "added_at": time.time()})
        # Rebuild lazily on the next search, not on every insertion or query.
        self._matrix_cache = None

    def search_similar(self, query_vector: list[float], top_k: int = 3) -> list[dict[str, Any]]:
        if top_k <= 0 or not self.vectors:
            return []
        query = self._normalize(query_vector)
        if self._matrix_cache is None:
            self._matrix_cache = np.vstack(self.vectors)
        scores = self._matrix_cache @ query
        count = min(top_k, scores.size)
        if count == scores.size:
            indices = np.argsort(scores)[::-1]
        else:
            candidates = np.argpartition(scores, scores.size - count)[-count:]
            indices = candidates[np.argsort(scores[candidates])[::-1]]
        return [
            {"score": float(scores[index]), "metadata": self.metadata[int(index)].copy()}
            for index in indices
        ]
