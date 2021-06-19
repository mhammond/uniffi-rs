A fixture for "External Types"

(todo: more words about what this multi-dir setup is demonstrating)

* `json` directory defines a `JSONObject` type, and refers to an external
  `Guid` type.

* `guid` directory same, but in reverse!

* `lib` directory is a "final" library which uses both of them. Only this has
  tests, as only it has both types in a final binary.
