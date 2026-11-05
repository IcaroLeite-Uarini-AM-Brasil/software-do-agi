"""Estimativa estatística de incerteza entre múltiplas saídas de um modelo.

O módulo não executa inferência nem garante que as amostras sejam independentes;
recebe distribuições de probabilidade produzidas pelo chamador.
"""
from __future__ import annotations

from typing import Any

import numpy as np


class EpistemicUncertaintyEstimator:
    def __init__(
        self,
        monte_carlo_samples: int | None = None,
        *,
        variance_threshold: float = 0.05,
        entropy_threshold: float = 1.5,
    ) -> None:
        if monte_carlo_samples is not None and monte_carlo_samples < 2:
            raise ValueError("monte_carlo_samples deve ser pelo menos 2")
        if not np.isfinite(variance_threshold) or variance_threshold < 0:
            raise ValueError("variance_threshold deve ser finito e não negativo")
        if not np.isfinite(entropy_threshold) or entropy_threshold < 0:
            raise ValueError("entropy_threshold deve ser finito e não negativo")
        self.mc_samples = monte_carlo_samples
        self.variance_threshold = variance_threshold
        self.entropy_threshold = entropy_threshold

    def estimate_uncertainty(self, model_outputs: list[list[float]]) -> dict[str, Any]:
        predictions = np.asarray(model_outputs, dtype=np.float64)
        if predictions.ndim != 2 or predictions.shape[0] < 2 or predictions.shape[1] < 2:
            raise ValueError("model_outputs deve conter ao menos duas amostras e duas classes")
        if self.mc_samples is not None and predictions.shape[0] != self.mc_samples:
            raise ValueError(f"eram esperadas {self.mc_samples} amostras")
        if not np.all(np.isfinite(predictions)):
            raise ValueError("as saídas devem conter apenas valores finitos")
        if np.any(predictions < 0):
            raise ValueError("as probabilidades não podem ser negativas")

        row_sums = predictions.sum(axis=1, keepdims=True)
        if np.any(row_sums <= 0) or not np.all(np.isfinite(row_sums)):
            raise ValueError("cada amostra deve ter soma de probabilidades positiva e finita")
        probabilities = predictions / row_sums

        predictive_variance = float(np.mean(np.var(probabilities, axis=0)))
        mean_prediction = probabilities.mean(axis=0)
        entropy = float(-np.sum(mean_prediction * np.log2(np.clip(mean_prediction, 1e-12, 1.0))))
        high_uncertainty = (
            predictive_variance > self.variance_threshold or entropy > self.entropy_threshold
        )
        return {
            "predictive_variance": round(predictive_variance, 6),
            "shannon_entropy": round(entropy, 6),
            "is_high_uncertainty": high_uncertainty,
        }


if __name__ == "__main__":
    estimator = EpistemicUncertaintyEstimator(monte_carlo_samples=5)
    sample_outputs = [
        [0.85, 0.10, 0.05],
        [0.80, 0.15, 0.05],
        [0.88, 0.08, 0.04],
        [0.82, 0.12, 0.06],
        [0.86, 0.09, 0.05],
    ]
    print(estimator.estimate_uncertainty(sample_outputs))
