from ext_types_lib import *
from ext_types_json import *
from ext_types_guid import *
import unittest

class TestExternalTypes(unittest.TestCase):
    def test_guid(self):
        guid = get_guid(None)
        self.assertEqual(type(guid), str)

        guid2 = get_guid(guid)
        self.assertEqual(guid, guid2)

    def test_json_object(self):
        j = get_json_object(None)
        self.assertEqual(type(j), dict)

        j2 = get_json_object(j)
        self.assertEqual(type(j2), dict)
        self.assertEqual(j, j2)

    def test_ext_types(self):
        # XXX - this fails, almost certainly because each generated python
        # file has its own RustBuffer?
        vals = get_lib_helper(None)

        self.assertEqual(type(vals.guid_helper.guid), str)
        self.assertEqual(vals.guid_helper.guid, "first-guid")
        self.assertEqual(type(vals.guid_helper.guids), list)
        self.assertEqual(vals.guid_helper.guids, ["second-guid", "third-guid"])

        self.assertEqual(type(vals.json_helper.json), dict)
        self.assertEqual(type(vals.json_helper.jsons), list)
        # first elt is a json array
        self.assertEqual(vals.json_helper.jsons[0], ["an", "array"])
        # second a plain int.
        self.assertEqual(vals.json_helper.jsons[1], 3)

        vals2 = get_lib_helper(vals)
        self.assertEqual(vals, vals2)

if __name__=='__main__':
    unittest.main()
