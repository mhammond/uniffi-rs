# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from ext_types_custom import *

class TestCallback(GuidCallback):
    def run(self, guid):
        self.saw_guid = guid
        return guid

def get_value_arg_issue(value):
    return f"""Failed to convert arg 'value':
Lifting custom type `ext_types_custom::Guid` from FFI type `alloc::string::String` failed at fixtures/ext-types/custom-types/src/lib.rs:101

Caused by:
    {value}"""

class TestCustomTypes(unittest.TestCase):
    def test_get_guid(self):
        self.assertEqual(get_guid(None), "NewGuid")
        self.assertEqual(get_guid("SomeGuid"), "SomeGuid")
        self.assertEqual(get_ouid(None), "Ouid")

    def test_guid_helper(self):
        helper = get_guid_helper(None)
        self.assertEqual(helper.guid, "first-guid")
        self.assertEqual(helper.guids, ["second-guid", "third-guid"])
        self.assertEqual(helper.maybe_guid, None)

    def test_get_guid_errors(self):
        # This is testing `get_guid` which never returns a result, so everything
        # is InternalError representing a panic.
        # The fixture hard-codes some Guid strings to return specific errors.
        with self.assertRaisesRegex(InternalError, get_value_arg_issue("The Guid is too short")):
            get_guid("")

        with self.assertRaisesRegex(InternalError, get_value_arg_issue("Something unexpected went wrong")):
            get_guid("unexpected")

        with self.assertRaisesRegex(InternalError, "guid value caused a panic!"):
            get_guid("panic")

    def test_try_get_guid_errors(self):
        # This is testing `try_get_guid()` which says it returns a result, so we
        # will get a mix of "expected" errors and panics.
        with self.assertRaises(GuidError.TooShort):
            try_get_guid("")

        with self.assertRaisesRegex(InternalError, get_value_arg_issue("Something unexpected went wrong")):
            try_get_guid("unexpected")

        with self.assertRaisesRegex(InternalError, "guid value caused a panic!"):
            try_get_guid("panic")

    def test_guid_callback(self):
        # Test that we can passing a guid from run_callback() to TestCallback.run() then back out

        test_callback = TestCallback()
        guid = run_callback(test_callback)
        self.assertEqual(guid, "callback-test-payload")
        self.assertEqual(test_callback.saw_guid, "callback-test-payload")

    def test_custom(self):
        self.assertEqual(get_handle_u8(None), 2)
        get_nested_object(InnerObject())

if __name__=='__main__':
    unittest.main()
