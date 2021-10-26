/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*

import uniffi.uniffi_ext_types_lib.*

// TODO: use an actual test runner.

val ct = getCombinedType(null);
assert(ct.uot.sval == "hello");
assert(ct.guid == "a-guid");
assert(ct.json, """{"hello":"there"}""")

val ct2 = getCombinedType(ct);
assert(ct == ct2)
