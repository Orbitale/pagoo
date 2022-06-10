
## Options

* `webhooks.[].matchers-strategy`: available values: 
  * `all`: The webhook HTTP request must match with all the matchers to be considered a match.
  * `one`: The webhook HTTP request must match with at least one matcher to be considered a match.
* `webhooks.[].matchers.[].match-json-body`: The JSON body to match against.<br>
  The body must be a valid JSON string.<br>
  **All** fields and subfields specified in this option have to match the HTTP request payload to be considered a match.<br>
  You can use [filters][#value-filters] to match fields in a different manner than simply "equals to".
* `webhooks.[].matchers.[].match-headers`: The HTTP headers to match against.<br>
  **All** headers specified in this option have to match the HTTP request headers to be considered a match.<br>
  You can use [filters][#value-filters] to match fields in a different manner than simply "equals to".

## Value filters

* `!regex:`: The value must not match the regular expression specified after the `:` character.

# Formats

## JSON format

[see json_sample.json](../samples/json_sample.json)

## Yaml format

[see yaml_sample.yaml](../samples/yaml_sample.yaml)
