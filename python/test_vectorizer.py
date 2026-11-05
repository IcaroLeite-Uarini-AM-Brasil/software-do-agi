import unittest

from vectorizer import VectorStoreEngine


class VectorStoreEngineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.store = VectorStoreEngine(embedding_dimension=3)

    def test_returns_nearest_document(self) -> None:
        self.store.add_document("AI", "a", [1.0, 0.0, 0.0])
        self.store.add_document("systems", "b", [0.0, 1.0, 0.0])
        result = self.store.search_similar([0.9, 0.1, 0.0], top_k=1)
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]["metadata"]["source"], "a")

    def test_rejects_bad_dimensions_and_non_finite_values(self) -> None:
        with self.assertRaises(ValueError):
            self.store.add_document("bad", "x", [1.0, 2.0])
        with self.assertRaises(ValueError):
            self.store.add_document("bad", "x", [1.0, float("nan"), 2.0])

    def test_rejects_zero_vectors(self) -> None:
        with self.assertRaises(ValueError):
            self.store.add_document("bad", "x", [0.0, 0.0, 0.0])

    def test_cache_is_invalidated_on_insert(self) -> None:
        self.store.add_document("first", "a", [1.0, 0.0, 0.0])
        self.store.search_similar([1.0, 0.0, 0.0])
        first_matrix = self.store._matrix_cache
        self.store.add_document("second", "b", [0.0, 1.0, 0.0])
        self.assertIsNone(self.store._matrix_cache)
        self.store.search_similar([0.0, 1.0, 0.0])
        self.assertIsNot(self.store._matrix_cache, first_matrix)

    def test_empty_store_and_non_positive_top_k(self) -> None:
        self.assertEqual(self.store.search_similar([1.0, 0.0, 0.0]), [])
        self.store.add_document("AI", "a", [1.0, 0.0, 0.0])
        self.assertEqual(self.store.search_similar([1.0, 0.0, 0.0], top_k=0), [])


if __name__ == "__main__":
    unittest.main()
