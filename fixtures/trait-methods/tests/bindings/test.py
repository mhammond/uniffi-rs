# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from trait_methods import *

class TestTraitMethods(unittest.TestCase):
    def test_str(self):
        m = TraitMethods("yo")
        self.assertEqual(str(m), "TraitMethods(yo)")

    def test_repr(self):
        m = TraitMethods("yo")
        self.assertEqual(repr(m), 'TraitMethods { val: "yo" }')

    def test_eq(self):
        m = TraitMethods("yo")
        self.assertEqual(m, TraitMethods("yo"))
        self.assertNotEqual(m, TraitMethods("yoyo"))

    def test_eq_wrong_type(self):
        m = TraitMethods("yo")
        self.assertNotEqual(m, 17)

    def test_hash(self):
        d = {}
        m = TraitMethods("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        a = TraitMethods("a")
        b = TraitMethods("b")
        self.assertLess(a, b)
        self.assertLessEqual(a, b)
        self.assertLessEqual(a, a)
        self.assertGreater(b, a)
        self.assertGreaterEqual(b, a)
        self.assertGreaterEqual(b, b)

class TestProcmacroTraitMethods(unittest.TestCase):
    def test_str(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(str(m), "ProcTraitMethods(yo)")

    def test_repr(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(repr(m), 'ProcTraitMethods { val: "yo" }')

    def test_eq(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(m, ProcTraitMethods("yo"))
        self.assertNotEqual(m, ProcTraitMethods("yoyo"))

    def test_eq(self):
        m = ProcTraitMethods("yo")
        self.assertNotEqual(m, 17)

    def test_hash(self):
        d = {}
        m = ProcTraitMethods("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        a = ProcTraitMethods("a")
        b = ProcTraitMethods("b")
        self.assertLess(a, b)
        self.assertLessEqual(a, b)
        self.assertLessEqual(a, a)
        self.assertGreater(b, a)
        self.assertGreaterEqual(b, a)
        self.assertGreaterEqual(b, b)

class TestTraitEnum(unittest.TestCase):
    def test_str(self):
        m = TraitEnum.S("yo")
        self.assertEqual(str(m), 'TraitEnum::S("yo")')

    def test_repr(self):
        m = TraitEnum.S("yo")
        self.assertEqual(repr(m), 'S("yo")')

    def test_eq(self):
        s1 = TraitEnum.S("1")
        s2 = TraitEnum.S("2")
        i1 = TraitEnum.I(1)
        i2 = TraitEnum.I(2)
        self.assertEqual(s1, s1)
        self.assertEqual(s1, s2)
        self.assertEqual(i1, i1)
        self.assertEqual(i1, i2)
        self.assertNotEqual(s1, i1)

    def test_eq_wrong_type(self):
        self.assertNotEqual(TraitEnum.S("1"), 17)

    def test_hash(self):
        d = {}
        m = TraitEnum.S("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        s1 = TraitEnum.S("1")
        i1 = TraitEnum.I(1)
        self.assertLess(s1, i1)
        self.assertLessEqual(s1, i1)

class TestUdlEnum(unittest.TestCase):
    def test_str(self):
        m = UdlEnum.S("yo")
        self.assertEqual(str(m), 'UdlEnum::S { s: "yo" }')

    def test_repr(self):
        m = UdlEnum.S("yo")
        self.assertEqual(repr(m), 'S { s: "yo" }')

    def test_eq(self):
        s1 = UdlEnum.S("1")
        s2 = UdlEnum.S("2")
        i1 = UdlEnum.I(1)
        i2 = UdlEnum.I(2)
        self.assertEqual(s1, s1)
        self.assertEqual(s1, s2)
        self.assertEqual(i1, i1)
        self.assertEqual(i1, i2)
        self.assertNotEqual(s1, i1)

    def test_eq_wrong_type(self):
        self.assertNotEqual(UdlEnum.S("1"), 17)

    def test_hash(self):
        d = {}
        m = UdlEnum.S("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        s1 = UdlEnum.S("1")
        i1 = UdlEnum.I(1)
        self.assertLess(s1, i1)
        self.assertLessEqual(s1, i1)

if __name__=='__main__':
    unittest.main()
