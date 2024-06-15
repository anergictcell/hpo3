import unittest

from pyhpo import Ontology
from pyhpo.set import HPOSet
from pyhpo.stats import EnrichmentModel
from pyhpo import annotations as an

# Number of terms in HPO Ontology
# grep "^\[Term\]$" pyhpo/data/hp.obo | wc -l
N_TERMS = 18961

# Number of genes in the annotation dataset
# cut -f4 pyhpo/data/phenotype_to_genes.txt | grep -v "^#" | grep -v "^gene_symbol" | sort -u | wc -l  # noqa: E501
# cut -f1 example_data/2024-03-06/genes_to_phenotype.txt | grep -v "^ncbi_gene_id" | sort -u | wc -l
N_GENES = 5075

# Number of OMIM diseases in the annotation dataset
# cut -f1,3 pyhpo/data/phenotype.hpoa | grep "^OMIM" | sort -u | cut -f2 | grep -v "NOT" | wc -l  # noqa: E501
N_OMIM = 8251

# Number of ORPHA diseases in the annotation dataset
# cut -f1,3 pyhpo/data/phenotype.hpoa | grep "^ORPHA" | sort -u | cut -f2 | grep -v "NOT" | wc -l  # noqa: E501
N_ORPHA = 4244


class IntegrationFullTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        Ontology()
        cls.terms = Ontology

    def test_terms_present(self):
        """
        These test will most likely need to be updated
        after every data update
        """
        assert len(self.terms) == N_TERMS, len(self.terms)

    def test_genes_associated(self):
        """
        These test will most likely need to be updated
        after every data update
        """
        assert len(self.terms.genes) == N_GENES, len(self.terms.genes)

    def test_omim_associated(self):
        """
        These test will most likely need to be updated
        after every data update
        """
        self.assertEqual(
            len(self.terms.omim_diseases),
            N_OMIM
        )

    def test_orpha_associated(self):
        """
        These test will most likely need to be updated
        after every data update
        """
        self.assertEqual(len(self.terms.orpha_diseases), N_ORPHA)

    def test_average_annotation_numbers(self):
        """
        These test will most likely need to be updated
        after every data update
        """
        genes = []
        omim = []
        for term in self.terms:
            genes.append(len(term.genes))
            omim.append(len(term.omim_diseases))

        assert sum(genes)/len(genes) > 36, sum(genes)/len(genes)
        assert sum(omim)/len(omim) > 29, sum(omim)/len(omim)

    def test_annotation_inheritance(self):
        for term in self.terms:
            lg = len(term.genes)
            lo = len(term.omim_diseases)

            for child in term.children:
                with self.subTest(t=term.id, c=child.id):
                    assert lg >= len(child.genes)
                    assert child.genes.issubset(term.genes)

                    assert lo >= len(child.omim_diseases)
                    assert child.omim_diseases.issubset(term.omim_diseases)

            for parent in term.parents:
                with self.subTest(t=term.id, c=parent.id):
                    assert lg <= len(parent.genes)
                    assert term.genes.issubset(parent.genes)

                    assert lo <= len(parent.omim_diseases)
                    assert term.omim_diseases.issubset(parent.omim_diseases)


    def test_relationships(self):
        for term in self.terms:
            for child in term.children:
                assert child.child_of(term)
                assert not child.parent_of(term)
                assert term.parent_of(child)
                assert not term.child_of(child)

            for parent in term.parents:
                assert not parent.child_of(term)
                assert parent.parent_of(term)
                assert not term.parent_of(parent)
                assert term.child_of(parent)

    def test_set(self):
        full_set = HPOSet.from_queries(
            [int(x) for x in self.terms]
        )

        self.assertEqual(
            len(full_set),
            len(self.terms)
        )

        phenoterms = full_set.remove_modifier()
        self.assertLess(
            len(phenoterms),
            len(full_set)
        )
        self.assertGreater(
            len(phenoterms),
            0
        )

        self.assertIn(
            self.terms[5],
            full_set
        )

        self.assertNotIn(
            self.terms[5],
            phenoterms
        )

    def test_gene_enrichment(self):
        hposet = HPOSet.from_queries('HP:0007401,HP:0010885'.split(','))
        gene_model = EnrichmentModel('gene')
        res = gene_model.enrichment('hypergeom', hposet)
        self.assertIsInstance(res, list)
        self.assertIn('item', res[0])
        self.assertIn('count', res[0])
        self.assertIn('enrichment', res[0])
        self.assertIsInstance(res[0]['item'], an.Gene)
        self.assertIsInstance(res[0]['count'], int)
        self.assertIsInstance(res[0]['enrichment'], float)

    def test_omim_enrichment(self):
        hposet = HPOSet.from_queries('HP:0007401,HP:0010885'.split(','))
        omim_model = EnrichmentModel('omim')
        res = omim_model.enrichment('hypergeom', hposet)
        self.assertIsInstance(res, list)
        self.assertIn('item', res[0])
        self.assertIn('count', res[0])
        self.assertIn('enrichment', res[0])
        self.assertIsInstance(res[0]['item'], an.Omim)
        self.assertIsInstance(res[0]['count'], int)
        self.assertIsInstance(res[0]['enrichment'], float)

    def test_orpha_enrichment(self):
        hposet = HPOSet.from_queries("HP:0007401,HP:0010885".split(","))
        orpha_model = EnrichmentModel('orpha')
        res = orpha_model.enrichment("hypergeom", hposet)
        self.assertIsInstance(res, list)
        self.assertIn("item", res[0])
        self.assertIn("count", res[0])
        self.assertIn("enrichment", res[0])
        self.assertIsInstance(res[0]["item"], an.Orpha)
        self.assertIsInstance(res[0]["count"], int)
        self.assertIsInstance(res[0]["enrichment"], float)
