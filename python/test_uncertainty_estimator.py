import unittest

import numpy as np

from uncertainty_estimator import EpistemicUncertaintyEstimator


class EpistemicUncertaintyEstimatorTests(unittest.TestCase):
    def test_confident_predictions_have_low_uncertainty(self) -> None:
        outputs = [[0.9, 0.1], [0.88, 0.12], [0.92, 0.08]]
        result = EpistemicUncertaintyEstimator(3).estimate_uncertainty(outputs)
        self.assertFalse(result["is_high_uncertainty"])
        self.assertGreaterEqual(result["shannon_entropy"], 0)

    def test_uniform_predictions_have_high_entropy(self) -> None:
        result = EpistemicUncertaintyEstimator(entropy_threshold=0.9).estimate_uncertainty(
            [[0.5, 0.5], [0.5, 0.5]]
        )
        self.assertTrue(result["is_high_uncertainty"])
        self.assertEqual(result["shannon_entropy"], 1.0)

    def test_detects_disagreement_across_samples(self) -> None:
        result = EpistemicUncertaintyEstimator(variance_threshold=0.05).estimate_uncertainty(
            [[1.0, 0.0], [0.0, 1.0]]
        )
        self.assertTrue(result["is_high_uncertainty"])
        self.assertEqual(result["predictive_variance"], 0.25)

    def test_rejects_malformed_or_invalid_probabilities(self) -> None:
        estimator = EpistemicUncertaintyEstimator()
        for bad in ([[1.0]], [[0.0, 0.0], [0.0, 0.0]], [[1.0, float("nan")], [0.5, 0.5]], [[-1.0, 2.0], [0.5, 0.5]]):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                estimator.estimate_uncertainty(bad)

    def test_rejects_sample_count_mismatch(self) -> None:
        with self.assertRaises(ValueError):
            EpistemicUncertaintyEstimator(3).estimate_uncertainty([[0.5, 0.5], [0.4, 0.6]])

    def test_non_numeric_input_raises_value_error(self) -> None:
        with self.assertRaises((ValueError, TypeError)):
            EpistemicUncertaintyEstimator().estimate_uncertainty([["x", "y"], ["a", "b"]])


if __name__ == "__main__":
    unittest.main()
