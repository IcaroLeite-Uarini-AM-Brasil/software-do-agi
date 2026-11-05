import time
from typing import Any

import numpy as np


class VectorStoreEngine:
    """Índice em memória pequeno, baseado em cosseno, para embeddings fornecidos pelo chamador.

    Esta classe não gera embeddings nem afirma ter compreensão semântica; uma
    integração com modelo pode fornecer vetores por meio de ``add_document``.
    """

    def __init__(self, embedding_dimension: int = 384) -> None:
        if embedding_dimension <= 0:
            raise ValueError("a dimensão do embedding deve ser positiva")
        self.dimension = embedding_dimension
        self.vectors: list[np.ndarray] = []
        self.metadata: list[dict[str, Any]] = []
        self._matrix_cache: np.ndarray | None = None

    def _normalize(self, values: list[float] | np.ndarray) -> np.ndarray:
        vector = np.asarray(values, dtype=np.float32)
        if vector.ndim != 1 or vector.shape[0] != self.dimension:
            raise ValueError(f"era esperado um vetor 1D de comprimento {self.dimension}")
        if not np.all(np.isfinite(vector)):
            raise ValueError("os valores do vetor devem ser finitos")
        norm = float(np.linalg.norm(vector))
        if norm == 0.0:
            raise ValueError("vetores nulos não podem ser indexados nem usados na busca")
        return vector / norm

    def add_document(self, text: str, source: str, raw_vector: list[float]) -> None:
        vector = self._normalize(raw_vector)
        self.vectors.append(vector)
        self.metadata.append({"text": text, "source": source, "added_at": time.time()})
        # Reconstrói a matriz de forma preguiçosa na próxima busca, não a cada inserção ou consulta.
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
