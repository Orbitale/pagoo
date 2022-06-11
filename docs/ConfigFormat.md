
## Options

* `webhooks`: Array of all webhooks that you want to listen to.<br>
  Webhooks options:
  * `name`: Name of the webhook. Used for analytics. 
  * `matchers-strategy`: available value
    * `all`: The webhook HTTP request must match with **all** the matchers to be considered a match.
    * `one`: The webhook HTTP request must match with **at least one** matcher to be considered a match.
  * `matchers`: an array of matchers that you can configure to determine how to execute your webhook action.<br>
    Matchers options:
    * `match-json-body`: The JSON body to match against.<br>
      The body must be a valid JSON string.<br>
      **All** fields and subfields specified in this option have to match the HTTP request payload to be considered a match.<br>
    * `match-headers`: The HTTP headers to match against.<br>
      **All** headers specified in this option have to match the HTTP request headers to be considered a match.<br>

# Formats

## JSON format

[see json_sample.json](../samples/json_sample.json)
