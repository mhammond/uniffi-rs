# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from error_types import *

class TestErrorTypes(unittest.TestCase):
    def test_normal_catch(self):
        try:
            oops()
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "oops")
           self.assertEqual(repr(e), "ErrorInterface { e: oops }")

        try:
            poops()
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "via a procmacro\n\nCaused by:\n    poops")

        try:
            poopse()
            self.fail("must fail")
        except EnumError as e:
           self.assertTrue(isinstance(e, EnumError.Oops))

    def test_interface(self):
        try:
            ErrorThrower(True).throw()
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "threw")

        try:
            ErrorThrower(False)
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "oops")

    # Check we can still call a function which returns an error (as opposed to one which throws it)
    def test_error_return(self):
        e = get_error("the error")
        self.assertEqual(e.chain(), ["the error"])
        self.assertEqual(repr(e), "ErrorInterface { e: the error }")
        self.assertEqual(str(e), "the error")

if __name__=='__main__':
    unittest.main()
